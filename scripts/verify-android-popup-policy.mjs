import { execFile } from 'node:child_process';
import { access } from 'node:fs/promises';
import { createServer } from 'node:http';
import { resolve } from 'node:path';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);
const packageName = 'com.titan.browser.debug';
const cdpPort = 9233;
const apkPath = resolve(process.env.TITAN_ANDROID_APK || 'android/app/build/outputs/apk/debug/app-debug.apk');
const skipInstall = process.argv.includes('--skip-install');
const keepData = process.argv.includes('--keep-data');
const wait = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds));

async function adb(...args) {
  const result = await execFileAsync('adb', args, { windowsHide: true });
  return result.stdout.trim();
}

async function waitForValue(read, predicate, label, timeout = 15_000) {
  const deadline = Date.now() + timeout;
  let value;
  while (Date.now() < deadline) {
    try {
      value = await read();
    } catch {
      await wait(100);
      continue;
    }
    if (predicate(value)) return value;
    await wait(100);
  }
  const detail = typeof value === 'string' && value.length > 500 ? `${value.slice(0, 500)}...` : value;
  throw new Error(`Timed out waiting for ${label}: ${JSON.stringify(detail)}`);
}

function fixturePage() {
  return `<!doctype html>
    <html>
      <body>
        <button id="popup" onclick="document.body.dataset.popupResult = window.open('/popup-destination', '_blank') === null ? 'blocked' : 'opened'">User popup</button>
        <a id="blank-link" href="/legitimate" target="_blank">Legitimate link</a>
        <button id="same-context" onclick="window.open('/same-context', '_self')">Same context</button>
        <iframe id="fixture-frame" name="fixture-frame"></iframe>
        <button id="named-frame" onclick="window.open('/frame-content', 'fixture-frame')">Named frame</button>
        <script>
          setTimeout(() => {
            try {
              document.body.dataset.automaticPopup = window.open('/automatic-popup', '_blank') ? 'opened' : 'blocked';
            } catch (error) {
              document.body.dataset.automaticPopup = 'blocked';
              document.body.dataset.automaticPopupError = error && error.name ? error.name : 'error';
            }
          }, 250);
        </script>
      </body>
    </html>`;
}

const server = createServer((request, response) => {
  response.setHeader('Cache-Control', 'no-store');
  response.setHeader('Content-Type', 'text/html; charset=utf-8');
  if (request.url === '/') response.end(fixturePage());
  else response.end(`<!doctype html><body data-path="${request.url}">${request.url}</body>`);
});

await new Promise((resolve, reject) => {
  server.once('error', reject);
  server.listen(0, '127.0.0.1', resolve);
});

const fixturePort = server.address().port;
const fixtureUrl = `http://127.0.0.1:${fixturePort}/`;
let socket;

try {
  if (!skipInstall) {
    await access(apkPath);
    await adb('install', '-r', apkPath);
  }
  if (!keepData) await adb('shell', 'pm', 'clear', packageName);
  await adb('logcat', '-c');
  await adb('reverse', `tcp:${fixturePort}`, `tcp:${fixturePort}`);
  await adb(
    'shell', 'am', 'start', '-W',
    '-a', 'android.intent.action.VIEW',
    '-d', fixtureUrl,
    '-n', `${packageName}/com.titan.browser.MainActivity`
  );

  const pid = await waitForValue(
    () => adb('shell', 'pidof', packageName),
    (value) => /^\d+$/.test(value),
    'Titan process'
  );
  await waitForValue(
    () => adb('shell', 'cat', '/proc/net/unix'),
    (value) => value.includes(`@webview_devtools_remote_${pid}`),
    'WebView debug socket'
  );
  await adb('forward', `tcp:${cdpPort}`, `localabstract:webview_devtools_remote_${pid}`);

  const baseUrl = `http://127.0.0.1:${cdpPort}`;
  const targets = await waitForValue(
    () => fetch(`${baseUrl}/json`).then((response) => response.json()),
    (value) => value.some((candidate) => candidate.type === 'page'),
    'WebView target'
  );
  const target = targets.find((candidate) => candidate.type === 'page' && JSON.parse(candidate.description).visible) ||
    targets.find((candidate) => candidate.type === 'page');

  socket = new WebSocket(target.webSocketDebuggerUrl);
  const pending = new Map();
  let nextId = 1;
  await new Promise((resolve, reject) => {
    socket.addEventListener('open', resolve, { once: true });
    socket.addEventListener('error', reject, { once: true });
  });
  socket.addEventListener('message', (event) => {
    const message = JSON.parse(String(event.data));
    if (!message.id) return;
    const request = pending.get(message.id);
    if (!request) return;
    pending.delete(message.id);
    if (message.error) request.reject(new Error(message.error.message));
    else request.resolve(message.result);
  });

  function call(method, params = {}) {
    const id = nextId++;
    socket.send(JSON.stringify({ id, method, params }));
    return new Promise((resolve, reject) => pending.set(id, { resolve, reject }));
  }

  async function evaluate(expression, userGesture = false) {
    const result = await call('Runtime.evaluate', {
      expression,
      awaitPromise: true,
      returnByValue: true,
      userGesture,
    });
    if (result.exceptionDetails) throw new Error(result.exceptionDetails.text);
    return result.result.value;
  }

  async function navigate(url) {
    await call('Page.navigate', { url });
    await waitForValue(
      () => evaluate(`({ path: location.pathname, readyState: document.readyState })`),
      (value) => value.readyState === 'complete' && new URL(url).pathname === value.path,
      `navigation to ${url}`
    );
    await wait(500);
  }

  async function tap(selector) {
    const point = await evaluate(`(() => {
      const rect = document.querySelector(${JSON.stringify(selector)}).getBoundingClientRect();
      return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
    })()`);
    await call('Input.dispatchTouchEvent', {
      type: 'touchStart',
      touchPoints: [{ ...point, radiusX: 1, radiusY: 1, force: 1, id: 1 }],
    });
    await call('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
  }

  await Promise.all([call('Page.enable'), call('Runtime.enable')]);
  await navigate(fixtureUrl);

  const targetCountBeforePopup = (await fetch(`${baseUrl}/json`).then((response) => response.json()))
    .filter((candidate) => candidate.type === 'page').length;
  const automaticPopup = await waitForValue(
    () => evaluate(`({
      result: document.body.dataset.automaticPopup,
      error: document.body.dataset.automaticPopupError || '',
      path: location.pathname
    })`),
    (value) => value.result === 'blocked',
    'automatic popup block'
  );
  const targetCountAfterAutomaticPopup = (await fetch(`${baseUrl}/json`).then((response) => response.json()))
    .filter((candidate) => candidate.type === 'page').length;

  await tap('#named-frame');
  const namedFrame = await waitForValue(
    () => evaluate(`document.getElementById('fixture-frame').contentDocument?.body?.dataset?.path || ''`),
    (value) => value === '/frame-content',
    'named-frame navigation'
  );

  await tap('#popup');
  const userPopupTargets = await waitForValue(
    () => fetch(`${baseUrl}/json`).then((response) => response.json()),
    (value) => value.some((candidate) => candidate.type === 'page' && candidate.url.endsWith('/popup-destination')),
    'user popup tab'
  );
  const sourceAfterUserPopup = await evaluate(`({ result: document.body.dataset.popupResult, path: location.pathname })`);
  await adb('shell', 'input', 'keyevent', '4');
  await wait(500);

  await tap('#blank-link');
  const blankLinkTargets = await waitForValue(
    () => fetch(`${baseUrl}/json`).then((response) => response.json()),
    (value) => value.some((candidate) => candidate.type === 'page' && candidate.url.endsWith('/legitimate')),
    'target-blank tab'
  );
  const sourceAfterBlankLink = await evaluate('location.pathname');
  await adb('shell', 'input', 'keyevent', '4');
  await wait(500);

  await tap('#same-context');
  const sameContext = await waitForValue(
    () => evaluate('location.pathname'),
    (value) => value === '/same-context',
    'same-context navigation'
  );

  const result = {
    automaticPopupBlocked: automaticPopup.result === 'blocked' && automaticPopup.path === '/',
    automaticPopupCreatedNoTab: targetCountAfterAutomaticPopup === targetCountBeforePopup,
    userPopupOpenedTab: sourceAfterUserPopup.result === 'opened' &&
      sourceAfterUserPopup.path === '/' &&
      userPopupTargets.some((candidate) => candidate.url.endsWith('/popup-destination')),
    legitimateBlankLinkOpenedTab: sourceAfterBlankLink === '/' &&
      blankLinkTargets.some((candidate) => candidate.url.endsWith('/legitimate')),
    namedFrameStillWorks: namedFrame === '/frame-content',
    explicitSameContextStillWorks: sameContext === '/same-context',
  };

  if (Object.values(result).some((value) => value !== true)) {
    throw new Error(`Popup policy verification failed: ${JSON.stringify(result)}`);
  }
  console.log(JSON.stringify(result, null, 2));
} finally {
  socket?.close();
  await adb('forward', '--remove', `tcp:${cdpPort}`).catch(() => {});
  await adb('reverse', '--remove', `tcp:${fixturePort}`).catch(() => {});
  server.close();
}
