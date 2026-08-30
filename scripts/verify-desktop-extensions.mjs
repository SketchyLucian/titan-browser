import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { createServer } from 'node:http';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const cdpPort = 9349;
const wait = (milliseconds) => new Promise((resolveWait) => setTimeout(resolveWait, milliseconds));

async function waitFor(read, predicate, label, timeout = 35_000) {
  const deadline = Date.now() + timeout;
  let value;
  while (Date.now() < deadline) {
    try {
      value = await read();
      if (predicate(value)) return value;
    } catch {
      // The browser or debug endpoint may still be starting.
    }
    await wait(200);
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
  async function evaluate(expression) {
    const id = nextId++;
    socket.send(JSON.stringify({ id, method: 'Runtime.evaluate', params: {
      expression,
      awaitPromise: true,
      returnByValue: true,
    } }));
    const result = await new Promise((resolveCall, reject) =>
      pending.set(id, { resolve: resolveCall, reject })
    );
    if (result.exceptionDetails) throw new Error(result.exceptionDetails.text);
    return result.result.value;
  }
  return { evaluate, close: () => socket.close() };
}

const server = createServer((request, response) => {
  response.setHeader('Content-Type', 'text/html; charset=utf-8');
  response.setHeader('Cache-Control', 'no-store');
  response.end(`<!doctype html>
<html>
<head><title>Extension Test Page</title></head>
<body>
  <h1>Extension Test Page</h1>
  <form id="login-form">
    <input type="text" name="username" id="username" placeholder="Username" autocomplete="username" />
    <input type="password" name="password" id="password" placeholder="Password" autocomplete="current-password" />
    <button type="submit">Log in</button>
  </form>
  <div id="ad-banner" class="adsbox ad-placement sponsor-banner">Sponsored Ad</div>
</body>
</html>`);
});

await new Promise((resolveListen, reject) => {
  server.once('error', reject);
  server.listen(0, '127.0.0.1', resolveListen);
});

const fixturePort = server.address().port;
const testUrl = `http://127.0.0.1:${fixturePort}/test`;
const root = await mkdtemp(join(tmpdir(), 'titan-ext-e2e-'));
const appDataDir = join(root, 'app-data');
const webviewDataDir = join(root, 'webview');
const executable = resolve(process.env.TITAN_TEST_EXECUTABLE || 'target/debug/titan-browser.exe');

const environment = {
  ...process.env,
  TITAN_WEBVIEW_DATA_DIR: webviewDataDir,
  TITAN_APP_DATA_DIR: appDataDir,
  TITAN_PROFILE_PORT: String(cdpPort),
};

let browser;

function startBrowser(args = []) {
  browser = spawn(executable, args, { env: environment, windowsHide: true, stdio: 'ignore' });
}

async function stopBrowser() {
  if (!browser || browser.exitCode !== null) return;
  browser.kill();
  await Promise.race([once(browser, 'exit'), wait(5_000)]);
  await wait(500);
}

try {
  const baseUrl = `http://127.0.0.1:${cdpPort}`;
  const listTargets = () => fetch(`${baseUrl}/json`).then((response) => response.json());

  startBrowser([testUrl]);

  const initialTargets = await waitFor(
    listTargets,
    (targets) => targets.some((target) => target.url === testUrl),
    'test page tab'
  );

  const headerTarget = initialTargets.find((target) => target.title === 'Titan Browser');
  if (!headerTarget) throw new Error('Titan header target not found');
  const header = await connect(headerTarget);

  console.log('1. Installing uBlock Origin Lite via live IPC...');
  await header.evaluate(`window.ipc.postMessage(JSON.stringify({ type: 'InstallExtension', id_or_url: 'ddkjiahejlhfcafbddmgiahcphecmpfh', source: 'chrome' }))`);

  console.log('2. Installing Bitwarden Password Manager via live IPC...');
  await header.evaluate(`window.ipc.postMessage(JSON.stringify({ type: 'InstallExtension', id_or_url: 'nngceckbapebfimnlniiiahkandclblb', source: 'chrome' }))`);

  // Open extensions manager page via IPC
  await wait(3000);
  await header.evaluate("window.ipc.postMessage(JSON.stringify({ type: 'OpenExtensions' }))");

  const settingsTarget = await waitFor(
    listTargets,
    (targets) => targets.some((target) => (target.title || '').includes('Settings')),
    'settings extensions tab'
  ).then((targets) => targets.find((target) => (target.title || '').includes('Settings')));

  const settings = await connect(settingsTarget);

  // Check that both uBlock Origin and Bitwarden are recognized and listed
  const extensionCards = await waitFor(
    () => settings.evaluate(`
      Array.from(document.querySelectorAll('.ext-card')).map(card => ({
        name: card.querySelector('.ext-card-name')?.textContent?.trim(),
        meta: card.querySelector('.ext-card-meta')?.textContent?.trim(),
        enabled: card.querySelector('input[type="checkbox"]')?.checked,
      }))
    `),
    (cards) => cards.length >= 2,
    '2 extension cards rendered in settings'
  );

  console.log('Detected extension cards in Titan Browser:', JSON.stringify(extensionCards, null, 2));

  const hasUblock = extensionCards.some((c) => (c.name || '').toLowerCase().includes('ublock') || (c.name || '').toLowerCase().includes('ubo'));
  const hasBitwarden = extensionCards.some((c) => (c.name || '').toLowerCase().includes('bitwarden'));

  console.log('3. Testing opening extension popup UI tab...');
  await header.evaluate("window.ipc.postMessage(JSON.stringify({ type: 'OpenExtensionPopup', id: 'ddkjiahejlhfcafbddmgiahcphecmpfh' }))");

  const extTabTarget = await waitFor(
    listTargets,
    (targets) => targets.some((target) => (target.url || '').startsWith('chrome-extension://')),
    'extension popup tab'
  ).then((targets) => targets.find((target) => (target.url || '').startsWith('chrome-extension://')));

  console.log('Opened extension UI target:', extTabTarget.url);

  if (extTabTarget.url.includes('google.com/search')) {
    throw new Error(`Extension UI redirected to Google Search: ${extTabTarget.url}`);
  }

  const result = {
    extensionsPageLoaded: true,
    totalExtensions: extensionCards.length,
    ublockOriginPresent: hasUblock,
    bitwardenPresent: hasBitwarden,
    allEnabled: extensionCards.every((c) => c.enabled),
    extensionUiLoadedDirectly: extTabTarget.url.startsWith('chrome-extension://'),
  };

  settings.close();
  header.close();

  console.log('Automated Extension Verification Result:', JSON.stringify(result, null, 2));

  if (!result.ublockOriginPresent || !result.bitwardenPresent || !result.extensionUiLoadedDirectly) {
    throw new Error('Verification failed: Extension UI did not load directly.');
  }

  console.log('✅ End-to-End Extension & Extension UI Verification PASSED!');
} finally {
  await stopBrowser();
  server.close();
  await rm(root, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
}
