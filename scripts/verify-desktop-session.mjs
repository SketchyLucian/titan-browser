import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { createServer } from 'node:http';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const cdpPort = 9342;
const wait = (milliseconds) => new Promise((resolveWait) => setTimeout(resolveWait, milliseconds));

async function waitFor(read, predicate, label, timeout = 15_000) {
  const deadline = Date.now() + timeout;
  let value;
  while (Date.now() < deadline) {
    try {
      value = await read();
      if (predicate(value)) return value;
    } catch {
      // The browser or debug endpoint may still be starting.
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
  const title = request.url === '/two' ? 'Session Two' : 'Session One';
  response.setHeader('Content-Type', 'text/html; charset=utf-8');
  response.setHeader('Cache-Control', 'no-store');
  response.end(`<!doctype html><title>${title}</title><body>${title}</body>`);
});
await new Promise((resolveListen, reject) => {
  server.once('error', reject);
  server.listen(0, '127.0.0.1', resolveListen);
});

const fixturePort = server.address().port;
const firstUrl = `http://127.0.0.1:${fixturePort}/one`;
const secondUrl = `http://127.0.0.1:${fixturePort}/two`;
const root = await mkdtemp(join(tmpdir(), 'titan-session-test-'));
const executable = resolve(process.env.TITAN_TEST_EXECUTABLE || 'target/debug/titan-browser.exe');
const environment = {
  ...process.env,
  TITAN_WEBVIEW_DATA_DIR: join(root, 'webview'),
  TITAN_APP_DATA_DIR: join(root, 'app-data'),
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
  startBrowser([firstUrl]);
  const initialTargets = await waitFor(
    listTargets,
    (targets) => targets.some((target) => target.url === firstUrl),
    'initial command-line tab'
  );
  const headerTarget = initialTargets.find((target) => target.title === 'Titan Browser');
  if (!headerTarget) throw new Error('Titan header target not found');
  const header = await connect(headerTarget);
  await header.evaluate(`window.ipc.postMessage(JSON.stringify({ type: 'NewTab', url: ${JSON.stringify(secondUrl)} }))`);
  await waitFor(
    listTargets,
    (targets) => targets.some((target) => target.url === secondUrl && target.title === 'Session Two'),
    'second session tab'
  );
  header.close();
  await wait(800);
  await stopBrowser();

  startBrowser();
  const restoredTargets = await waitFor(
    listTargets,
    (targets) => targets.some((target) => target.url === firstUrl) &&
      targets.some((target) => target.url === secondUrl),
    'restored tabs'
  );
  const restoredHeaderTarget = restoredTargets.find((target) => target.title === 'Titan Browser');
  if (!restoredHeaderTarget) throw new Error('Restored Titan header target not found');
  const restoredHeader = await connect(restoredHeaderTarget);
  await restoredHeader.evaluate("window.ipc.postMessage(JSON.stringify({ type: 'OpenHistory' }))");
  const historyTarget = await waitFor(
    listTargets,
    (targets) => targets.find((target) => target.title === 'History'),
    'history page'
  ).then((targets) => targets.find((target) => target.title === 'History'));
  const history = await connect(historyTarget);
  const historyUrls = await waitFor(
    () => history.evaluate("Array.from(document.querySelectorAll('.url')).map(element => element.textContent)"),
    (urls) => urls.includes(firstUrl) && urls.includes(secondUrl),
    'history entries'
  );

  const result = {
    restoredFirstTab: restoredTargets.some((target) => target.url === firstUrl),
    restoredSecondTab: restoredTargets.some((target) => target.url === secondUrl),
    historyContainsBoth: historyUrls.includes(firstUrl) && historyUrls.includes(secondUrl),
  };
  history.close();
  restoredHeader.close();
  if (Object.values(result).some((value) => value !== true)) {
    throw new Error(`Session verification failed: ${JSON.stringify(result)}`);
  }
  console.log(JSON.stringify(result, null, 2));
} finally {
  await stopBrowser();
  server.close();
  await rm(root, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
}
