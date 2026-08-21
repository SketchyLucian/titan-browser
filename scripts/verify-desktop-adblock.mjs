import { writeFile } from 'node:fs/promises';

const port = Number(process.argv[2] || 9335);
const screenshotPath = process.argv[3];
const baseUrl = `http://127.0.0.1:${port}`;
const testUrl = 'https://adblock.turtlecute.org/';
const youtubeUrl = 'https://www.youtube.com/watch?v=jNQXAC9IVRw';

const wait = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds));

async function listTargets() {
  const response = await fetch(`${baseUrl}/json`);
  if (!response.ok) throw new Error(`CDP target list failed with ${response.status}`);
  return response.json();
}

async function connect(target) {
  const socket = new WebSocket(target.webSocketDebuggerUrl);
  const pending = new Map();
  let nextId = 1;

  await new Promise((resolve, reject) => {
    socket.addEventListener('open', resolve, { once: true });
    socket.addEventListener('error', reject, { once: true });
  });

  socket.addEventListener('message', (event) => {
    const message = JSON.parse(String(event.data));
    const pendingCall = pending.get(message.id);
    if (!pendingCall) return;
    pending.delete(message.id);
    if (message.error) pendingCall.reject(new Error(message.error.message));
    else pendingCall.resolve(message.result);
  });

  function call(method, params = {}) {
    const id = nextId++;
    socket.send(JSON.stringify({ id, method, params }));
    return new Promise((resolve, reject) => pending.set(id, { resolve, reject }));
  }

  async function evaluate(expression) {
    const result = await call('Runtime.evaluate', {
      expression,
      awaitPromise: true,
      returnByValue: true,
    });
    if (result.exceptionDetails) {
      throw new Error(result.exceptionDetails.text || 'Evaluation failed');
    }
    return result.result.value;
  }

  return { call, evaluate, close: () => socket.close() };
}

async function findNewPageTarget(previousIds, urlPrefix, label) {
  const deadline = Date.now() + 15_000;
  while (Date.now() < deadline) {
    const targets = await listTargets();
    const target = targets.find((candidate) =>
      candidate.type === 'page' &&
      !previousIds.has(candidate.id) &&
      candidate.url.startsWith(urlPrefix)
    );
    if (target) return target;
    await wait(100);
  }
  throw new Error(`${label} target did not appear`);
}

async function waitForResult(client) {
  const deadline = Date.now() + 45_000;
  let result;

  while (Date.now() < deadline) {
    result = await client.evaluate(`(() => {
      const text = document.body?.innerText || '';
      const lines = text.split('\\n').map((line) => line.trim()).filter(Boolean);
      return {
        readyState: document.readyState,
        title: document.title,
        url: location.href,
        score: lines.find((line) => /^\\d+%$/.test(line)) || null,
        total: lines.find((line) => /^Total\\s*:\\s*\\d+/i.test(line)) || null,
        blocked: lines.find((line) => /^\\d+\\s+blocked$/i.test(line)) || null,
        notBlocked: lines.find((line) => /^\\d+\\s+not blocked$/i.test(line)) || null,
        evidence: lines.filter((line) => /^(\\d+%|Total\\s*:|\\d+\\s+(not )?blocked$)/i.test(line))
      };
    })()`);

    if (result.total && result.blocked && result.notBlocked) {
      const blocked = Number(result.blocked.match(/^\d+/)?.[0] || 0);
      const total = Number(result.total.match(/\d+/)?.[0] || 0);
      result.score = total > 0 ? `${Math.round((blocked / total) * 100)}%` : result.score;
      return result;
    }
    await wait(250);
  }

  throw new Error(`Timed out waiting for a complete result: ${JSON.stringify(result)}`);
}

async function main() {
  const initialTargets = await listTargets();
  const headerTarget = initialTargets.find((target) => target.title === 'Titan Browser');
  if (!headerTarget) throw new Error('Titan header target not found');

  const header = await connect(headerTarget);
  const previousIds = new Set(initialTargets.map((target) => target.id));
  await header.evaluate(`window.ipc.postMessage(JSON.stringify({ type: 'NewTab', url: '${testUrl}' }))`);

  const testTarget = await findNewPageTarget(previousIds, testUrl, 'Adblock test');
  const page = await connect(testTarget);
  await page.call('Page.enable');
  const result = await waitForResult(page);

  if (screenshotPath) {
    const screenshot = await page.call('Page.captureScreenshot', {
      format: 'png',
      captureBeyondViewport: false,
    });
    await writeFile(screenshotPath, Buffer.from(screenshot.data, 'base64'));
    result.screenshotPath = screenshotPath;
  }

  const beforeYoutube = await listTargets();
  const beforeYoutubeIds = new Set(beforeYoutube.map((target) => target.id));
  await header.evaluate(`window.ipc.postMessage(JSON.stringify({ type: 'NewTab', url: '${youtubeUrl}' }))`);
  const youtubeTarget = await findNewPageTarget(beforeYoutubeIds, 'https://www.youtube.com/', 'YouTube');
  const youtube = await connect(youtubeTarget);
  const youtubeDeadline = Date.now() + 25_000;
  let youtubeState;
  while (Date.now() < youtubeDeadline) {
    youtubeState = await youtube.evaluate(`(() => {
      const video = document.querySelector('video');
      const blockerCss = Array.from(document.querySelectorAll('style[id^="titan-"]'))
        .map((style) => style.textContent || '')
        .join('\\n');
      const parsedProbe = JSON.parse('{"adPlacements":[{"probe":true}],"playerAds":[{"probe":true}],"adSlots":[{"probe":true}],"videoDetails":{"title":"content-preserved"}}');
      const playerResponse = window.ytInitialPlayerResponse;
      return {
        readyState: document.readyState,
        title: document.title,
        url: location.href,
        playerResponseSanitized:
          !Object.prototype.hasOwnProperty.call(parsedProbe, 'adPlacements') &&
          !Object.prototype.hasOwnProperty.call(parsedProbe, 'playerAds') &&
          !Object.prototype.hasOwnProperty.call(parsedProbe, 'adSlots') &&
          parsedProbe.videoDetails?.title === 'content-preserved',
        skipUiParentsVisible:
          !blockerCss.includes('.video-ads') &&
          !blockerCss.includes('.ytp-ad-module'),
        initialResponseHasAds: !!(
          playerResponse?.adPlacements?.length ||
          playerResponse?.playerAds?.length ||
          playerResponse?.adSlots?.length
        ),
        actualAdShowing: !!document.querySelector('.ad-showing, .ad-interrupting'),
        skipButtonCount: document.querySelectorAll('button.ytp-ad-skip-button, button.ytp-ad-skip-button-modern, button.ytp-skip-ad-button, .ytp-ad-skip-button-slot button').length,
        video: video ? {
          readyState: video.readyState,
          currentTime: Number(video.currentTime.toFixed(2)),
          duration: Number.isFinite(video.duration) ? Number(video.duration.toFixed(2)) : null,
          paused: video.paused
        } : null
      };
    })()`);
    if (
      youtubeState.readyState === 'complete' &&
      youtubeState.playerResponseSanitized &&
      youtubeState.skipUiParentsVisible &&
      youtubeState.video?.readyState >= 1
    ) break;
    await wait(250);
  }
  result.youtube = youtubeState;
  youtube.close();

  const beforeSettings = await listTargets();
  await header.evaluate(`window.ipc.postMessage(JSON.stringify({ type: 'OpenAdblock' }))`);
  const settingsDeadline = Date.now() + 10_000;
  let settingsTarget;
  while (Date.now() < settingsDeadline && !settingsTarget) {
    const targets = await listTargets();
    settingsTarget = targets.find((candidate) =>
      candidate.type === 'page' &&
      candidate.id !== testTarget.id &&
      candidate.title !== 'Titan Browser' &&
      (candidate.title.includes('AdBlock') || candidate.title.includes('Settings'))
    );
    if (!settingsTarget) await wait(100);
  }

  if (settingsTarget) {
    const settings = await connect(settingsTarget);
    result.nativeActivity = await settings.evaluate(`(() => ({
      blockedRequestCount: document.getElementById('statAdsBlocked')?.textContent?.trim() || null,
      activityCount: document.getElementById('adblockActivityCountBadge')?.textContent?.trim() || null,
      recentActivity: Array.from(document.querySelectorAll('#adblockActivityLogList .log-item'))
        .slice(0, 5)
        .map((element) => element.innerText.trim())
    }))()`);
    settings.close();
  }

  page.close();
  header.close();
  console.log(JSON.stringify(result, null, 2));
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
