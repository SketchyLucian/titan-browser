import { execFile } from 'node:child_process';
import { access } from 'node:fs/promises';
import { createServer } from 'node:http';
import { resolve } from 'node:path';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);
const wait = (milliseconds) => new Promise((resolveWait) => setTimeout(resolveWait, milliseconds));
const args = process.argv.slice(2);

function option(name, fallback) {
  const prefix = `--${name}=`;
  return args.find((arg) => arg.startsWith(prefix))?.slice(prefix.length) ?? fallback;
}

const adbPath = option('adb', process.env.ADB || 'adb');
const packageName = option('package', 'com.titan.browser.debug');
const activityName = option('activity', 'com.titan.browser.MainActivity');
const apkPath = resolve(option('apk', 'android/app/build/outputs/apk/debug/app-debug.apk'));
const serial = option('serial', process.env.ANDROID_SERIAL || '');
const cdpPort = Number(option('cdp-port', '9334'));
const skipInstall = args.includes('--skip-install');
const keepData = args.includes('--keep-data');
const componentName = `${packageName}/${activityName}`;
const cookieName = `titan_login_${Date.now()}`;
const downloadName = `titan-android-daily-driver-${Date.now()}.txt`;
const expectedDownloadBody = `Titan Android daily-driver download ${downloadName}\n`;
const deviceDownloadPath = `/sdcard/Download/${downloadName}`;
const requests = [];

function fixturePage() {
  return `<!doctype html>
    <html>
      <head>
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <style>
          body { margin: 0; font-family: sans-serif; background: #f8fafc; color: #111827; }
          main { padding: 24px; }
          button, a, input[type=file] {
            box-sizing: border-box;
            display: block;
            width: 100%;
            min-height: 112px;
            margin: 0 0 18px;
            padding: 28px;
            border: 2px solid #334155;
            border-radius: 10px;
            background: #ffffff;
            color: #111827;
            font: 700 28px sans-serif;
            text-align: left;
          }
          a { text-decoration: none; }
        </style>
      </head>
      <body>
        <main>
          <button id="login" onclick="document.cookie='${cookieName}=ok; Path=/; SameSite=Lax'; location.href='/cookie-check'">Login cookie</button>
          <a id="download" href="/download">Download file</a>
          <input id="upload" type="file" aria-label="Upload file">
          <button id="location" onclick="navigator.geolocation.getCurrentPosition(() => {}, () => {})">Location permission</button>
          <button id="media" onclick="navigator.mediaDevices.getUserMedia({ video: true, audio: true }).catch(() => {})">Camera microphone permission</button>
        </main>
      </body>
    </html>`;
}

function page(title, body = '') {
  return `<!doctype html><meta name="viewport" content="width=device-width, initial-scale=1"><title>${title}</title><body>${body}</body>`;
}

const server = createServer((request, response) => {
  const url = new URL(request.url || '/', 'http://127.0.0.1');
  requests.push({
    path: url.pathname,
    cookie: request.headers.cookie || '',
    at: Date.now(),
  });

  response.setHeader('Cache-Control', 'no-store');
  if (url.pathname === '/download') {
    response.setHeader('Content-Type', 'text/plain; charset=utf-8');
    response.setHeader('Content-Disposition', `attachment; filename="${downloadName}"`);
    response.end(expectedDownloadBody);
    return;
  }

  response.setHeader('Content-Type', 'text/html; charset=utf-8');
  if (url.pathname === '/' || url.pathname === '/start') response.end(fixturePage());
  else if (url.pathname === '/cookie-check') response.end(page('Cookie check', 'cookie checked'));
  else if (url.pathname === '/cookie-check-after-restart') response.end(page('Cookie after restart', 'cookie checked'));
  else if (url.pathname === '/restore') response.end(page('Restore target', 'restore target'));
  else response.end(page('Titan fixture', url.pathname));
});

async function runAdb(adbArgs, options = {}) {
  const finalArgs = serial ? ['-s', serial, ...adbArgs] : adbArgs;
  try {
    const result = await execFileAsync(adbPath, finalArgs, {
      windowsHide: true,
      timeout: options.timeout ?? 30_000,
      maxBuffer: options.maxBuffer ?? 10 * 1024 * 1024,
    });
    return {
      ok: true,
      stdout: result.stdout.trim(),
      stderr: result.stderr.trim(),
      command: `adb ${finalArgs.join(' ')}`,
    };
  } catch (error) {
    const result = {
      ok: false,
      stdout: String(error.stdout || '').trim(),
      stderr: String(error.stderr || error.message || '').trim(),
      command: `adb ${finalArgs.join(' ')}`,
    };
    if (options.allowFailure) return result;
    throw new Error(`${result.command} failed\n${result.stderr || result.stdout}`);
  }
}

async function adb(...adbArgs) {
  return (await runAdb(adbArgs)).stdout;
}

async function waitFor(read, predicate, label, timeout = 20_000) {
  const deadline = Date.now() + timeout;
  let value;
  while (Date.now() < deadline) {
    try {
      value = await read();
      if (predicate(value)) return value;
    } catch {
      // Device UI, DownloadManager, WebView devtools, or the local server may still be settling.
    }
    await wait(250);
  }
  const detail = typeof value === 'string' && value.length > 500 ? `${value.slice(0, 500)}...` : value;
  throw new Error(`Timed out waiting for ${label}: ${JSON.stringify(detail)}`);
}

function requestCount(path) {
  return requests.filter((request) => request.path === path).length;
}

function sawRequest(path, predicate = () => true) {
  return requests.some((request) => request.path === path && predicate(request));
}

async function chooseDevice() {
  const devices = (await runAdb(['devices', '-l'], { timeout: 15_000 })).stdout
    .split(/\r?\n/)
    .slice(1)
    .map((line) => line.trim())
    .filter(Boolean)
    .filter((line) => /\sdevice\s/.test(line));
  if (serial) return devices.find((line) => line.startsWith(serial)) || serial;
  if (devices.length !== 1) {
    throw new Error(`Expected exactly one ADB device. Found ${devices.length}: ${devices.join(' | ') || 'none'}`);
  }
  return devices[0];
}

function safeJsonParse(value) {
  try {
    return JSON.parse(value);
  } catch {
    return undefined;
  }
}

function visibleTarget(target) {
  return safeJsonParse(target.description)?.visible === true;
}

async function connectToWebView(baseUrl) {
  const pid = await waitFor(
    () => adb('shell', 'pidof', packageName),
    (value) => /^\d+$/.test(value),
    'Titan process'
  );
  await waitFor(
    () => adb('shell', 'cat', '/proc/net/unix'),
    (value) => value.includes(`@webview_devtools_remote_${pid}`),
    'WebView debug socket'
  );
  await adb('forward', `tcp:${cdpPort}`, `localabstract:webview_devtools_remote_${pid}`);

  const devtoolsUrl = `http://127.0.0.1:${cdpPort}`;
  const targets = await waitFor(
    () => fetch(`${devtoolsUrl}/json`).then((response) => response.json()),
    (value) => value.some((target) => target.type === 'page'),
    'WebView target'
  );
  const target = targets.find((candidate) =>
    candidate.type === 'page' && candidate.url.startsWith(baseUrl)
  ) || targets.find((candidate) => candidate.type === 'page' && visibleTarget(candidate)) ||
    targets.find((candidate) => candidate.type === 'page');

  if (!globalThis.WebSocket) {
    throw new Error('Node.js WebSocket support is required. Use Node.js 22 or newer.');
  }

  const socket = new WebSocket(target.webSocketDebuggerUrl);
  const pending = new Map();
  let nextId = 1;
  await new Promise((resolveOpen, reject) => {
    socket.addEventListener('open', resolveOpen, { once: true });
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
    return new Promise((resolveCall, reject) => pending.set(id, { resolve: resolveCall, reject }));
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
    await waitFor(
      () => evaluate('({ href: location.href, readyState: document.readyState })'),
      (value) => value.readyState === 'complete' && value.href === url,
      `navigation to ${url}`
    );
  }

  async function tap(selector) {
    const point = await evaluate(`(() => {
      const element = document.querySelector(${JSON.stringify(selector)});
      if (!element) throw new Error('Missing selector: ${selector}');
      const rect = element.getBoundingClientRect();
      return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
    })()`);
    await call('Input.dispatchTouchEvent', {
      type: 'touchStart',
      touchPoints: [{ ...point, radiusX: 1, radiusY: 1, force: 1, id: 1 }],
    });
    await call('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
  }

  await Promise.all([call('Page.enable'), call('Runtime.enable')]);
  return {
    navigate,
    tap,
    close: () => socket.close(),
  };
}

async function dumpUi() {
  await adb('shell', 'uiautomator', 'dump', '/sdcard/titan-window.xml');
  return adb('exec-out', 'cat', '/sdcard/titan-window.xml');
}

function nodeAttribute(node, name) {
  return node.match(new RegExp(`${name}="([^"]*)"`))?.[1] || '';
}

function boundsForText(xml, text) {
  const expected = text.toLocaleLowerCase();
  for (const match of xml.matchAll(/<node\b[^>]*>/g)) {
    const node = match[0];
    if (nodeAttribute(node, 'text').toLocaleLowerCase() !== expected) continue;
    const bounds = nodeAttribute(node, 'bounds').match(/\[(\d+),(\d+)\]\[(\d+),(\d+)\]/);
    if (!bounds) continue;
    const [, left, top, right, bottom] = bounds.map(Number);
    return {
      x: Math.round((left + right) / 2),
      y: Math.round((top + bottom) / 2),
    };
  }
  return undefined;
}

async function tapDialogButton(text) {
  const xml = await dumpUi();
  const point = boundsForText(xml, text);
  if (!point) throw new Error(`Could not find dialog button "${text}"`);
  await adb('shell', 'input', 'tap', String(point.x), String(point.y));
}

async function startViewIntent(url) {
  await adb(
    'shell', 'am', 'start', '-W',
    '-a', 'android.intent.action.VIEW',
    '-d', url,
    '-n', componentName
  );
}

await new Promise((resolveListen, reject) => {
  server.once('error', reject);
  server.listen(0, '127.0.0.1', resolveListen);
});

const fixturePort = server.address().port;
const baseUrl = `http://127.0.0.1:${fixturePort}`;
const startUrl = `${baseUrl}/start`;
let connection;
let cleanedDownload = false;

try {
  const device = await chooseDevice();
  if (!skipInstall) {
    await access(apkPath);
    await adb('install', '-r', apkPath);
  }
  if (!keepData) await adb('shell', 'pm', 'clear', packageName);
  await adb('logcat', '-c');
  await adb('reverse', `tcp:${fixturePort}`, `tcp:${fixturePort}`);

  await startViewIntent(startUrl);
  await waitFor(
    () => sawRequest('/start'),
    Boolean,
    'VIEW intent load'
  );
  connection = await connectToWebView(baseUrl);

  await connection.tap('#login');
  await waitFor(
    () => sawRequest('/cookie-check', (request) => request.cookie.includes(`${cookieName}=ok`)),
    Boolean,
    'login cookie round trip'
  );

  await connection.navigate(startUrl);
  await connection.tap('#download');
  await waitFor(
    () => sawRequest('/download'),
    Boolean,
    'download request'
  );
  await waitFor(
    async () => runAdb(['shell', 'cat', deviceDownloadPath], { allowFailure: true }),
    (result) => result.ok && result.stdout === expectedDownloadBody.trim(),
    'downloaded file on device',
    45_000
  );

  await connection.navigate(startUrl);
  await connection.tap('#upload');
  await waitFor(
    dumpUi,
    (xml) => !xml.includes(`package="${packageName}"`) &&
      (/documentsui/i.test(xml) || /resolver/i.test(xml) || /package="android"/i.test(xml)),
    'Android file chooser'
  );
  await adb('shell', 'input', 'keyevent', '4');
  await waitFor(
    dumpUi,
    (xml) => xml.includes(`package="${packageName}"`),
    'return from file chooser'
  );

  await connection.navigate(startUrl);
  await connection.tap('#location');
  await waitFor(
    dumpUi,
    (xml) => xml.includes('Allow location?') && xml.includes('wants to use your location.'),
    'location permission prompt'
  );
  await tapDialogButton('Block');

  await connection.navigate(startUrl);
  await connection.tap('#media');
  await waitFor(
    dumpUi,
    (xml) => xml.includes('Allow camera and microphone?') ||
      xml.includes('Allow camera?') ||
      xml.includes('Allow microphone?'),
    'camera or microphone permission prompt'
  );
  await tapDialogButton('Block');

  await connection.navigate(`${baseUrl}/restore`);
  await waitFor(
    () => requestCount('/restore') >= 1,
    Boolean,
    'initial restore target load'
  );
  const restoreHits = requestCount('/restore');
  await adb('shell', 'input', 'keyevent', '3');
  await wait(1_000);
  await adb('shell', 'am', 'force-stop', packageName);
  connection.close();
  connection = undefined;
  await adb('forward', '--remove', `tcp:${cdpPort}`).catch(() => {});
  await adb('shell', 'am', 'start', '-W', '-n', componentName);
  await waitFor(
    () => requestCount('/restore') > restoreHits,
    Boolean,
    'session restore after process restart',
    30_000
  );

  await startViewIntent(`${baseUrl}/cookie-check-after-restart`);
  await waitFor(
    () => sawRequest('/cookie-check-after-restart', (request) =>
      request.cookie.includes(`${cookieName}=ok`)
    ),
    Boolean,
    'login cookie after restart'
  );

  const crashLog = await adb('logcat', '-d', '-v', 'brief', 'AndroidRuntime:E', '*:S');
  if (crashLog.includes('FATAL EXCEPTION') || crashLog.includes(packageName)) {
    throw new Error(`Android runtime crash detected:\n${crashLog}`);
  }

  cleanedDownload = (await runAdb(['shell', 'rm', deviceDownloadPath], { allowFailure: true })).ok;

  console.log(JSON.stringify({
    device,
    installedDebugApk: !skipInstall,
    viewIntentHandled: true,
    loginCookieWorks: true,
    loginCookiePersistsAfterRestart: true,
    downloadSavedByDownloadManager: true,
    uploadChooserOpened: true,
    locationPromptShown: true,
    cameraMicrophonePromptShown: true,
    sessionRestoredAfterRestart: true,
    noAndroidRuntimeCrash: true,
    cleanedDownload,
  }, null, 2));
} finally {
  connection?.close();
  await runAdb(['forward', '--remove', `tcp:${cdpPort}`], { allowFailure: true }).catch(() => {});
  await runAdb(['reverse', '--remove', `tcp:${fixturePort}`], { allowFailure: true }).catch(() => {});
  if (!cleanedDownload) {
    await runAdb(['shell', 'rm', deviceDownloadPath], { allowFailure: true }).catch(() => {});
  }
  server.close();
}
