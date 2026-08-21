import { spawn } from 'node:child_process';
import { createServer } from 'node:http';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const cdpPort = 9341;
const wait = (milliseconds) => new Promise((resolveWait) => setTimeout(resolveWait, milliseconds));

async function waitFor(read, predicate, label, timeout = 15_000) {
  const deadline = Date.now() + timeout;
  let value;
  while (Date.now() < deadline) {
    try {
      value = await read();
      if (predicate(value)) return value;
    } catch {
      // The browser or target may still be starting.
    }
    await wait(100);
  }
  throw new Error(`Timed out waiting for ${label}: ${JSON.stringify(value)}`);
}

async function connect(target) {
  const socket = new WebSocket(target.webSocketDebuggerUrl);
  const pending = new Map();
  let nextId = 1;
  await new Promise((resolveOpen, reject) => {
    socket.addEventListener('open', resolveOpen, { once: true });
    socket.addEventListener('error', reject, { once: true });
  });
  socket.addEventListener('message', (event) => {
    const message = JSON.parse(String(event.data));
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

  return { evaluate, close: () => socket.close() };
}

const server = createServer((request, response) => {
  response.setHeader('Content-Type', 'text/html; charset=utf-8');
  response.setHeader('Cache-Control', 'no-store');
  if (request.url === '/popup') {
    response.end(`<!doctype html><title>Popup Target</title><script>
      document.addEventListener('DOMContentLoaded', () => {
        document.body.dataset.hasOpener = String(window.opener !== null);
        window.opener?.postMessage('titan-popup-opener-ok', '*');
      });
    </script><body>Popup target</body>`);
  } else {
    response.end(`<!doctype html><title>Popup Source</title>
      <button id="openPopup" onclick="window.open('/popup', '_blank')">Open popup</button>
      <script>window.addEventListener('message', event => document.body.dataset.message = event.data);</script>`);
  }
});

await new Promise((resolveListen, reject) => {
  server.once('error', reject);
  server.listen(0, '127.0.0.1', resolveListen);
});

const fixturePort = server.address().port;
const fixtureUrl = `http://127.0.0.1:${fixturePort}/`;
const profile = await mkdtemp(join(tmpdir(), 'titan-popup-test-'));
const executable = resolve(process.env.TITAN_TEST_EXECUTABLE || 'target/debug/titan-browser.exe');
const browser = spawn(executable, [fixtureUrl], {
  env: {
    ...process.env,
    TITAN_WEBVIEW_DATA_DIR: profile,
    TITAN_APP_DATA_DIR: join(profile, 'app-data'),
    TITAN_PROFILE_PORT: String(cdpPort),
  },
  windowsHide: true,
  stdio: 'ignore',
});

try {
  const baseUrl = `http://127.0.0.1:${cdpPort}`;
  const listTargets = () => fetch(`${baseUrl}/json`).then((response) => response.json());
  const sourceTarget = await waitFor(
    listTargets,
    (targets) => targets.find((target) => target.url === fixtureUrl),
    'popup source target'
  ).then((targets) => targets.find((target) => target.url === fixtureUrl));
  const source = await connect(sourceTarget);
  const headerTarget = (await listTargets()).find((target) => target.title === 'Titan Browser');
  if (!headerTarget) throw new Error('Titan header target not found');
  const header = await connect(headerTarget);
  await source.evaluate("window.dispatchEvent(new KeyboardEvent('keydown', { key: 'l', ctrlKey: true }))");
  const contentShortcutFocusedAddressBar = await waitFor(
    () => header.evaluate('document.activeElement?.id'),
    (value) => value === 'urlInput',
    'content shortcut address-bar focus'
  ).then(() => true);
  await source.evaluate("document.getElementById('openPopup').click()", true);

  const popupTarget = await waitFor(
    listTargets,
    (targets) => targets.find((target) => target.url.endsWith('/popup')),
    'popup target'
  ).then((targets) => targets.find((target) => target.url.endsWith('/popup')));
  const popup = await connect(popupTarget);
  const popupHasOpener = await waitFor(
    () => popup.evaluate('document.body.dataset.hasOpener'),
    (value) => value === 'true',
    'popup opener relationship'
  );
  const openerMessage = await waitFor(
    () => source.evaluate('document.body.dataset.message'),
    (value) => value === 'titan-popup-opener-ok',
    'popup-to-opener message'
  );

  await source.evaluate("document.cookie = 'regularCookie=visible; path=/'; localStorage.setItem('titanClearTest', 'present')");
  const beforePrivate = new Set((await listTargets()).map((target) => target.id));
  await header.evaluate("window.ipc.postMessage(JSON.stringify({ type: 'NewPrivateTab' }))");
  const privateTarget = await waitFor(
    listTargets,
    (targets) => targets.find((target) => target.type === 'page' && !beforePrivate.has(target.id)),
    'private tab target'
  ).then((targets) => targets.find((target) => target.type === 'page' && !beforePrivate.has(target.id)));
  const privateTab = await connect(privateTarget);
  await waitFor(
    () => privateTab.evaluate("typeof window.ipc?.postMessage"),
    (value) => value === 'function',
    'private tab IPC bridge'
  );
  await privateTab.evaluate(`window.ipc.postMessage(JSON.stringify({ type: 'Navigate', url: ${JSON.stringify(fixtureUrl)} }))`);
  await waitFor(
    () => privateTab.evaluate('location.href'),
    (value) => value === fixtureUrl,
    'private tab navigation'
  );
  const regularCookieHidden = !(await privateTab.evaluate('document.cookie')).includes('regularCookie=visible');
  await privateTab.evaluate("document.cookie = 'privateCookie=temporary; path=/'");
  await privateTab.evaluate("window.dispatchEvent(new KeyboardEvent('keydown', { key: 'w', ctrlKey: true }))");
  await waitFor(
    listTargets,
    (targets) => !targets.some((target) => target.id === privateTarget.id),
    'private tab close'
  );
  privateTab.close();

  const beforeSecondPrivate = new Set((await listTargets()).map((target) => target.id));
  await header.evaluate("window.ipc.postMessage(JSON.stringify({ type: 'NewPrivateTab' }))");
  const secondPrivateTarget = await waitFor(
    listTargets,
    (targets) => targets.find((target) => target.type === 'page' && !beforeSecondPrivate.has(target.id)),
    'second private tab target'
  ).then((targets) => targets.find((target) => target.type === 'page' && !beforeSecondPrivate.has(target.id)));
  const secondPrivate = await connect(secondPrivateTarget);
  await waitFor(
    () => secondPrivate.evaluate("typeof window.ipc?.postMessage"),
    (value) => value === 'function',
    'second private tab IPC bridge'
  );
  await secondPrivate.evaluate(`window.ipc.postMessage(JSON.stringify({ type: 'Navigate', url: ${JSON.stringify(fixtureUrl)} }))`);
  await waitFor(
    () => secondPrivate.evaluate('location.href'),
    (value) => value === fixtureUrl,
    'second private tab navigation'
  );
  const privateCookieCleared = !(await secondPrivate.evaluate('document.cookie')).includes('privateCookie=temporary');
  await header.evaluate(`window.ipc.postMessage(JSON.stringify({
    type: 'ClearBrowsingData',
    cookies: true,
    cache: true,
    local_storage: true
  }))`);
  const nativeClearRemovedRegularData = await waitFor(
    async () => ({
      cookie: await source.evaluate("document.cookie.includes('regularCookie=visible')"),
      storage: await source.evaluate("localStorage.getItem('titanClearTest')"),
    }),
    (value) => value.cookie === false && value.storage === null,
    'native regular-profile data clear'
  ).then(() => true);

  popup.close();
  source.close();
  secondPrivate.close();
  header.close();
  const result = {
    popupHasOpener: popupHasOpener === 'true',
    openerMessageReceived: openerMessage === 'titan-popup-opener-ok',
    regularCookieHidden,
    privateCookieCleared,
    nativeClearRemovedRegularData,
    contentShortcutFocusedAddressBar,
  };
  if (Object.values(result).some((value) => value !== true)) {
    throw new Error(`Desktop compatibility verification failed: ${JSON.stringify(result)}`);
  }
  console.log(JSON.stringify(result, null, 2));
} finally {
  browser.kill();
  server.close();
  await wait(500);
  await rm(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
}
