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

  console.log('3. Installing Dark Reader via live IPC...');
  await header.evaluate(`window.ipc.postMessage(JSON.stringify({ type: 'InstallExtension', id_or_url: 'eimadpbcbfnmbkopoojfekhnkhdbieeh', source: 'chrome' }))`);

  // Open extensions manager page via IPC
  await wait(5000);
  await header.evaluate("window.ipc.postMessage(JSON.stringify({ type: 'OpenExtensions' }))");

  const settingsTarget = await waitFor(
    async () => {
      const targets = await listTargets();
      for (const target of targets) {
        if (target.type !== 'page') continue;
        if ((target.title || '').includes('Settings')) return target;
        if ((target.title || '') === 'Titan Browser' || target.url === testUrl) continue;
        const page = await connect(target);
        try {
          const isSettingsPage = await page.evaluate(`
            document.title.includes('Settings') || !!document.querySelector('.settings-container')
          `);
          if (isSettingsPage) return target;
        } finally {
          page.close();
        }
      }
      return null;
    },
    (target) => !!target,
    'settings extensions tab'
  );

  const settings = await connect(settingsTarget);

  // Check that installed extensions are recognized and listed
  const extensionCards = await waitFor(
    () => settings.evaluate(`
      Array.from(document.querySelectorAll('.ext-card')).map(card => ({
        name: card.querySelector('.ext-card-name')?.textContent?.trim(),
        meta: card.querySelector('.ext-card-meta')?.textContent?.trim(),
        enabled: card.querySelector('input[type="checkbox"]')?.checked,
      }))
    `),
    (cards) => cards.length >= 3,
    '3 extension cards rendered in settings'
  );

  console.log('Detected extension cards in Titan Browser:', JSON.stringify(extensionCards, null, 2));

  const hasUblock = extensionCards.some((c) => (c.name || '').toLowerCase().includes('ublock') || (c.name || '').toLowerCase().includes('ubo'));
  const hasBitwarden = extensionCards.some((c) => (c.name || '').toLowerCase().includes('bitwarden'));
  const hasDarkReader = extensionCards.some((c) => (c.name || '').toLowerCase().includes('dark reader'));

  const isOnboardingUrl = (url) =>
    /^https:\/\/bitwarden\.com\/browser-start\/?/i.test(url || '') ||
    /^https:\/\/darkreader\.org\/help\//i.test(url || '');

  async function closeExtensionOnboardingTabs() {
    await waitFor(
      () => header.evaluate(`(() => {
        const tabs = window.__TITAN_BROWSER_STATE__?.tabs || [];
        for (const tab of tabs) {
          if (
            /^https:\\/\\/bitwarden\\.com\\/browser-start\\/?/i.test(tab.url || '') ||
            /^https:\\/\\/darkreader\\.org\\/help\\//i.test(tab.url || '')
          ) {
            window.ipc.postMessage(JSON.stringify({ type: 'CloseTab', tab_id: tab.id }));
          }
        }
        return tabs.map((tab) => ({ id: tab.id, url: tab.url }));
      })()`),
      (tabs) => tabs.every((tab) => !isOnboardingUrl(tab.url)),
      'extension onboarding tabs closed',
      10_000
    );
    await wait(300);
  }

  async function readTabState() {
    return header.evaluate(`({
      activeId: window.__TITAN_BROWSER_STATE__?.activeTabId ?? window.__TITAN_BROWSER_STATE__?.active_tab_id,
      count: window.__TITAN_BROWSER_STATE__?.tabs?.length ?? 0,
      urls: window.__TITAN_BROWSER_STATE__?.tabs?.map((tab) => tab.url) ?? []
    })`);
  }

  function assertNoDelayedViewportRebound(label, samples, dimension) {
    let lowWaterMark = samples[0]?.[dimension] ?? 0;
    let sawShrink = false;
    for (const sample of samples) {
      const value = sample[dimension];
      if (value < lowWaterMark - 4) {
        lowWaterMark = value;
        sawShrink = true;
      } else if (sawShrink && value > lowWaterMark + 4) {
        throw new Error(
          `${label} popup ${dimension} rebounded after shrinking: ${JSON.stringify(samples)}`
        );
      }
    }
  }

  async function assertAnchoredExtensionPopup({ storeId, titlePattern, label, textPattern, minWidth = 240, maxWidth = 360 }) {
    console.log(`4. Testing ${label} anchored extension popup UI...`);
    await closeExtensionOnboardingTabs();
    const storeIdLiteral = JSON.stringify(storeId);
    const metadata = await waitFor(
      () => header.evaluate(`
      window.__TITAN_BROWSER_STATE__?.extensions
        ?.find((ext) => ext.id === ${storeIdLiteral} || ext.runtime_id === ${storeIdLiteral})
    `),
      (ext) => ext && ext.popup_page && (ext.runtime_id || ext.id),
      `${label} extension runtime metadata`
    );
    const runtimeId = metadata.runtime_id || metadata.id;
    const expectedPopupUrl = `chrome-extension://${runtimeId}/${metadata.popup_page.replace(/^\/+/, '')}`;
    const titlePatternLiteral = JSON.stringify(titlePattern);

    await header.evaluate(`
      (() => {
        const titlePattern = new RegExp(${titlePatternLiteral}, 'i');
        const button = Array.from(document.querySelectorAll('.toolbar-ext-btn'))
          .find((button) => titlePattern.test(button.title || ''));
        if (!button) throw new Error('Toolbar button not found for ${label}');
        button.click();
      })()
    `);

    const headerSurface = await waitFor(
      () => header.evaluate(`({
        dropdownDisplay: getComputedStyle(document.getElementById('extensionsDropdown')).display,
        headerHeight: window.innerHeight
      })`),
      (surface) => surface.dropdownDisplay === 'none' && surface.headerHeight <= 85,
      `${label} dropdown dismissed while popup opens`,
      10_000
    );

    const popupTarget = await waitFor(
      listTargets,
      (targets) => targets.some((target) => target.url === expectedPopupUrl),
      `${label} extension popup WebView`
    ).then((targets) => targets.find((target) => target.url === expectedPopupUrl));

    console.log(`Opened ${label} extension UI target:`, popupTarget.url);

    const tabStateAfterPopup = await readTabState();
    const popupCreatedTitanTab = tabStateAfterPopup.urls.includes(expectedPopupUrl);

    if (popupCreatedTitanTab) {
      throw new Error(`${label} popup opened as a Titan tab instead of an anchored popup: ${JSON.stringify(tabStateAfterPopup)}`);
    }

    const extensionPopup = await connect(popupTarget);
    const expectedTextPattern = new RegExp(textPattern, 'i');
    const popupDocument = await waitFor(
      () => extensionPopup.evaluate(`({
        href: window.location.href,
        readyState: document.readyState,
        elementCount: document.body?.querySelectorAll('*').length || 0,
        text: document.body?.innerText?.slice(0, 240) || ''
      })`),
      (documentState) =>
        (documentState.href === expectedPopupUrl || documentState.href.startsWith(`${expectedPopupUrl}#`)) &&
        documentState.readyState !== 'loading' &&
        documentState.elementCount > 0 &&
        expectedTextPattern.test(documentState.text),
      `${label} extension popup document`
    );

    if (/ERR_|can't be reached|not found/i.test(popupDocument.text)) {
      throw new Error(`${label} popup rendered an error page: ${popupDocument.text}`);
    }

    const readPopupViewport = () =>
      extensionPopup.evaluate(`({
        width: window.innerWidth,
        height: window.innerHeight
      })`);
    const popupViewportState = await waitFor(
      async () => {
        const first = await readPopupViewport();
        await wait(350);
        const second = await readPopupViewport();
        return {
          width: second.width,
          height: second.height,
          widthDelta: Math.abs(second.width - first.width),
          heightDelta: Math.abs(second.height - first.height),
        };
      },
      (viewport) =>
        viewport.width >= minWidth &&
        viewport.width < maxWidth &&
        viewport.widthDelta < 4 &&
        viewport.heightDelta < 4,
      `${label} stable extension popup viewport`,
      12_000
    );
    const viewportSamples = [
      { width: popupViewportState.width, height: popupViewportState.height, elapsedMs: 0 },
    ];
    const viewportWatchStarted = Date.now();
    while (Date.now() - viewportWatchStarted < 9_000) {
      await wait(500);
      viewportSamples.push({
        ...(await readPopupViewport()),
        elapsedMs: Date.now() - viewportWatchStarted,
      });
    }
    assertNoDelayedViewportRebound(label, viewportSamples, 'width');
    assertNoDelayedViewportRebound(label, viewportSamples, 'height');

    const finalSamples = viewportSamples.slice(-6);
    const popupViewport = finalSamples.at(-1);
    if (popupViewport.width < minWidth || popupViewport.width >= maxWidth) {
      throw new Error(`${label} popup final viewport width was out of range: ${JSON.stringify(viewportSamples)}`);
    }
    const widthRange = Math.max(...finalSamples.map((sample) => sample.width)) - Math.min(...finalSamples.map((sample) => sample.width));
    const heightRange = Math.max(...finalSamples.map((sample) => sample.height)) - Math.min(...finalSamples.map((sample) => sample.height));
    if (widthRange >= 4 || heightRange >= 4) {
      throw new Error(`${label} popup viewport did not stay stable after open: ${JSON.stringify(viewportSamples)}`);
    }

    await header.evaluate("window.ipc.postMessage(JSON.stringify({ type: 'CloseExtensionPopup' }))");
    extensionPopup.close();
    await wait(300);

    return {
      url: popupTarget.url,
      headerSurface,
      documentLoaded: popupDocument.elementCount > 0,
      viewport: popupViewport,
      viewportStable: true,
      openedOutsideTabStack: !popupCreatedTitanTab,
    };
  }

  async function assertExtensionsDropdownVisibleAfterNativePopup() {
    console.log('5. Testing Extensions menu replaces an open native popup...');
    await closeExtensionOnboardingTabs();
    const bitwardenStoreId = 'nngceckbapebfimnlniiiahkandclblb';
    const metadata = await waitFor(
      () => header.evaluate(`
        window.__TITAN_BROWSER_STATE__?.extensions
          ?.find((ext) => ext.id === '${bitwardenStoreId}' || ext.runtime_id === '${bitwardenStoreId}')
      `),
      (ext) => ext && ext.popup_page && (ext.runtime_id || ext.id),
      'Bitwarden extension runtime metadata for dropdown replacement'
    );
    const runtimeId = metadata.runtime_id || metadata.id;
    const expectedPopupUrl = `chrome-extension://${runtimeId}/${metadata.popup_page.replace(/^\/+/, '')}`;

    await header.evaluate(`
      (() => {
        const button = Array.from(document.querySelectorAll('.toolbar-ext-btn'))
          .find((button) => /Bitwarden/i.test(button.title || ''));
        if (!button) throw new Error('Bitwarden toolbar button not found');
        button.click();
      })()
    `);
    await waitFor(
      listTargets,
      (targets) => targets.some((target) => (target.url || '').startsWith(expectedPopupUrl)),
      'Bitwarden native popup before opening Extensions menu'
    );

    await header.evaluate("document.getElementById('extensionsBtn')?.click()");

    const surface = await waitFor(
      () => header.evaluate(`(() => {
        const dropdown = document.getElementById('extensionsDropdown');
        const list = document.getElementById('extensionsDropdownList');
        const body = document.getElementById('extensionsDropdownBody');
        const dropdownRect = dropdown?.getBoundingClientRect();
        const bodyRect = body?.getBoundingClientRect();
        const listRect = list?.getBoundingClientRect();
        return {
          dropdownDisplay: dropdown ? getComputedStyle(dropdown).display : '',
          headerHeight: window.innerHeight,
          dropdownHeight: dropdownRect?.height || 0,
          bodyHeight: bodyRect?.height || 0,
          listHeight: listRect?.height || 0,
          itemCount: list?.querySelectorAll('.ext-drop-item').length || 0,
        };
      })()`),
      (state) =>
        state.dropdownDisplay !== 'none' &&
        state.headerHeight >= 400 &&
        state.dropdownHeight > 120 &&
        state.bodyHeight > 60 &&
        state.listHeight > 60 &&
        state.itemCount >= 3,
      'full Extensions dropdown after native popup'
    );
    await waitFor(
      listTargets,
      (targets) => !targets.some((target) => (target.url || '').startsWith(expectedPopupUrl)),
      'native extension popup closed when Extensions menu opens',
      10_000
    );
    await header.evaluate("document.getElementById('closeExtDropdownBtn')?.click()");
    await wait(300);

    return surface;
  }

  const ublockPopup = await assertAnchoredExtensionPopup({
    storeId: 'ddkjiahejlhfcafbddmgiahcphecmpfh',
    titlePattern: 'uBlock|uBO',
    label: 'uBlock',
    textPattern: '\\S',
  });
  const bitwardenPopup = await assertAnchoredExtensionPopup({
    storeId: 'nngceckbapebfimnlniiiahkandclblb',
    titlePattern: 'Bitwarden',
    label: 'Bitwarden',
    textPattern: 'bitwarden|Anmelden|Log in',
    minWidth: 280,
    maxWidth: 380,
  });
  const darkReaderPopup = await assertAnchoredExtensionPopup({
    storeId: 'eimadpbcbfnmbkopoojfekhnkhdbieeh',
    titlePattern: 'Dark Reader',
    label: 'Dark Reader',
    textPattern: 'Dark Reader',
    minWidth: 260,
    maxWidth: 340,
  });
  const extensionsDropdownAfterPopup = await assertExtensionsDropdownVisibleAfterNativePopup();

  const result = {
    extensionsPageLoaded: true,
    totalExtensions: extensionCards.length,
    ublockOriginPresent: hasUblock,
    bitwardenPresent: hasBitwarden,
    darkReaderPresent: hasDarkReader,
    allEnabled: extensionCards.every((c) => c.enabled),
    ublockPopup,
    bitwardenPopup,
    darkReaderPopup,
    extensionsDropdownAfterPopup,
  };

  settings.close();
  header.close();

  console.log('Automated Extension Verification Result:', JSON.stringify(result, null, 2));

  if (
    !result.ublockOriginPresent ||
    !result.bitwardenPresent ||
    !result.darkReaderPresent ||
    !result.ublockPopup.openedOutsideTabStack ||
    !result.bitwardenPopup.openedOutsideTabStack ||
    !result.darkReaderPopup.openedOutsideTabStack ||
    result.extensionsDropdownAfterPopup.itemCount < 3
  ) {
    throw new Error('Verification failed: Extension UI did not load as an anchored popup.');
  }

  console.log('✅ End-to-End Extension & Extension UI Verification PASSED!');
} finally {
  await stopBrowser();
  server.close();
  await rm(root, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
}
