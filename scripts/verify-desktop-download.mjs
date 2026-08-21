import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { createServer } from 'node:http';
import { mkdtemp, readFile, rm, stat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const cdpPort = 9343;
const expectedBody = 'Titan desktop download acceptance\n';
const wait = (milliseconds) => new Promise((resolveWait) => setTimeout(resolveWait, milliseconds));

async function waitFor(read, predicate, label, timeout = 20_000) {
  const deadline = Date.now() + timeout;
  let value;
  while (Date.now() < deadline) {
    try {
      value = await read();
      if (predicate(value)) return value;
    } catch {
      // The browser, target, file, or persistent record may still be starting.
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

  async function evaluate(expression, userGesture = false) {
    const id = nextId++;
    socket.send(JSON.stringify({
      id,
      method: 'Runtime.evaluate',
      params: { expression, awaitPromise: true, returnByValue: true, userGesture },
    }));
    const result = await new Promise((resolveCall, reject) =>
      pending.set(id, { resolve: resolveCall, reject })
    );
    if (result.exceptionDetails) throw new Error(result.exceptionDetails.text);
    return result.result.value;
  }

  return { evaluate, close: () => socket.close() };
}

const server = createServer((request, response) => {
  if (request.url === '/file') {
    response.setHeader('Content-Type', 'text/plain');
    response.setHeader('Content-Disposition', 'attachment; filename="titan-download-test.txt"');
    response.end(expectedBody);
    return;
  }
  response.setHeader('Content-Type', 'text/html; charset=utf-8');
  response.end('<!doctype html><title>Download Test</title><a id="download" href="/file">Download</a>');
});

await new Promise((resolveListen, reject) => {
  server.once('error', reject);
  server.listen(0, '127.0.0.1', resolveListen);
});

const fixtureUrl = `http://127.0.0.1:${server.address().port}/`;
const testRoot = await mkdtemp(join(tmpdir(), 'titan-download-test-'));
const profile = join(testRoot, 'webview');
const appData = join(testRoot, 'app-data');
const downloads = join(testRoot, 'downloads');
const expectedFile = join(downloads, 'titan-download-test.txt');
const executable = resolve(process.env.TITAN_TEST_EXECUTABLE || 'target/debug/titan-browser.exe');
const browser = spawn(executable, [fixtureUrl], {
  env: {
    ...process.env,
    TITAN_WEBVIEW_DATA_DIR: profile,
    TITAN_APP_DATA_DIR: appData,
    TITAN_DOWNLOAD_DIR: downloads,
    TITAN_PROFILE_PORT: String(cdpPort),
  },
  windowsHide: true,
  stdio: 'ignore',
});

try {
  const targets = await waitFor(
    async () => (await fetch(`http://127.0.0.1:${cdpPort}/json`)).json(),
    (items) => items.some((item) => item.url === fixtureUrl),
    'download fixture target'
  );
  const fixtureTarget = targets.find((item) => item.url === fixtureUrl);
  const connection = await connect(fixtureTarget);
  await waitFor(
    () => connection.evaluate('Boolean(document.querySelector("#download"))'),
    Boolean,
    'download link'
  );
  await connection.evaluate('document.querySelector("#download").click()', true);

  await waitFor(
    async () => (await stat(expectedFile)).size,
    (size) => size === Buffer.byteLength(expectedBody),
    'downloaded file'
  );
  const body = await readFile(expectedFile, 'utf8');
  if (body !== expectedBody) throw new Error(`Unexpected downloaded contents: ${JSON.stringify(body)}`);

  const records = await waitFor(
    async () => JSON.parse(await readFile(join(appData, 'downloads.json'), 'utf8')),
    (items) => items.some((item) =>
      item.status === 'complete' && item.file_path?.endsWith('titan-download-test.txt')
    ),
    'completed persistent download record'
  );
  const record = records.find((item) => item.file_path?.endsWith('titan-download-test.txt'));
  console.log(JSON.stringify({
    fileDownloaded: true,
    fileContentsVerified: true,
    persistentStatus: record.status,
    isolatedDownloadDirectory: true,
  }, null, 2));
  connection.close();
} finally {
  if (browser.exitCode === null) {
    browser.kill();
    await Promise.race([once(browser, 'exit'), wait(5_000)]);
  }
  server.close();
  await rm(testRoot, { recursive: true, force: true, maxRetries: 10, retryDelay: 200 });
}
