const port = Number(process.argv[2] || 9223);
const baseUrl = `http://127.0.0.1:${port}`;

async function listTargets() {
  const response = await fetch(`${baseUrl}/json`);
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
    if (!message.id) return;
    const waiter = pending.get(message.id);
    if (!waiter) return;
    pending.delete(message.id);
    if (message.error) waiter.reject(new Error(message.error.message));
    else waiter.resolve(message.result);
  });

  function call(method, params = {}) {
    const id = nextId++;
    socket.send(JSON.stringify({ id, method, params }));
    return new Promise((resolve, reject) => pending.set(id, { resolve, reject }));
  }

  async function evaluate(expression, awaitPromise = false) {
    const result = await call('Runtime.evaluate', {
      expression,
      awaitPromise,
      returnByValue: true,
    });
    if (result.exceptionDetails) {
      throw new Error(result.exceptionDetails.text || 'Evaluation failed');
    }
    return result.result.value;
  }

  return { call, evaluate, close: () => socket.close() };
}

function percentile(values, quantile) {
  if (!values.length) return 0;
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.min(sorted.length - 1, Math.ceil(sorted.length * quantile) - 1)];
}

const sampleFramesExpression = String.raw`
  new Promise((resolve) => {
    const intervals = [];
    const start = performance.now();
    let previous;
    function frame(now) {
      if (previous !== undefined) intervals.push(now - previous);
      previous = now;
      if (now - start < 700) requestAnimationFrame(frame);
      else resolve({ intervals, start: performance.timeOrigin + start });
    }
    requestAnimationFrame(frame);
    document.getElementById('settingsBtn').click();
  })
`;

const sampleSettingsFramesExpression = String.raw`
  new Promise((resolve) => {
    const intervals = [];
    let startedAt;
    let previous;
    function frame(now) {
      if (startedAt === undefined) startedAt = now;
      if (previous !== undefined) intervals.push(now - previous);
      previous = now;
      if (now - startedAt < 350) requestAnimationFrame(frame);
      else resolve(intervals);
    }
    requestAnimationFrame(frame);
  })
`;

async function waitForSettingsTarget(previousIds, startedAt, existingTarget, existingClient) {
  const deadline = Date.now() + 5000;
  let target = existingTarget;
  let client = existingClient;
  let appearedMs = target ? 0 : undefined;

  while (Date.now() < deadline) {
    if (!target) {
      const targets = await listTargets();
      target = targets.find((candidate) =>
        candidate.type === 'page' &&
        !previousIds.has(candidate.id) &&
        candidate.title !== 'Titan Browser'
      );
      if (target) appearedMs = performance.now() - startedAt;
    }

    if (target && !client) client = await connect(target);
    if (target && client) {
      const state = await client.evaluate(`({
        ready: document.readyState,
        active: !!document.querySelector('.tab-view.active'),
        focused: document.hasFocus(),
        navigation: performance.getEntriesByType('navigation')[0]?.toJSON(),
        paints: performance.getEntriesByType('paint').map((entry) => entry.toJSON()),
        animations: document.getAnimations().map((animation) => ({
          duration: animation.effect?.getTiming().duration,
          currentTime: animation.currentTime,
          playState: animation.playState
        }))
      })`);
      if (state.ready === 'complete' && state.active && state.focused) {
        return {
          targetId: target.id,
          appearedMs,
          readyMs: performance.now() - startedAt,
          state,
          client,
        };
      }
    }
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
  client?.close();
  throw new Error('Settings target did not become active');
}

async function main() {
  const initialTargets = await listTargets();
  const headerTarget = initialTargets.find((target) => target.title === 'Titan Browser');
  if (!headerTarget) throw new Error('Titan header target not found');
  const header = await connect(headerTarget);
  const runs = [];

  for (let run = 1; run <= 5; run++) {
    const before = await listTargets();
    const previousIds = new Set(before.map((target) => target.id));
    const existingSettingsTarget = before.find((target) => target.title.startsWith('Settings'));
    const existingSettingsClient = existingSettingsTarget
      ? await connect(existingSettingsTarget)
      : undefined;
    const settingsFramesPromise = existingSettingsClient
      ? existingSettingsClient.evaluate(sampleSettingsFramesExpression, true)
      : undefined;
    const startedAt = performance.now();
    const headerFramesPromise = header.evaluate(sampleFramesExpression, true);
    const settingsPromise = waitForSettingsTarget(
      previousIds,
      startedAt,
      existingSettingsTarget,
      existingSettingsClient
    );
    const [headerFrames, settings] = await Promise.all([headerFramesPromise, settingsPromise]);
    const settingsFrames = settingsFramesPromise
      ? await settingsFramesPromise
      : await settings.client.evaluate(sampleSettingsFramesExpression, true);

    const summarize = (intervals) => ({
      frames: intervals.length,
      missed: intervals.filter((value) => value > 20).length,
      p50: Number(percentile(intervals, 0.5).toFixed(2)),
      p95: Number(percentile(intervals, 0.95).toFixed(2)),
      max: Number(Math.max(...intervals).toFixed(2)),
    });

    runs.push({
      run,
      targetAppearedMs: Number(settings.appearedMs.toFixed(2)),
      settingsReadyMs: Number(settings.readyMs.toFixed(2)),
      domContentLoadedMs: Number((settings.state.navigation?.domContentLoadedEventEnd || 0).toFixed(2)),
      loadMs: Number((settings.state.navigation?.loadEventEnd || 0).toFixed(2)),
      firstPaintMs: Number((settings.state.paints?.find((entry) => entry.name === 'first-paint')?.startTime || 0).toFixed(2)),
      animationAtReady: settings.state.animations?.[0] || null,
      headerFrames: summarize(headerFrames.intervals),
      settingsFrames: summarize(settingsFrames),
    });

    settings.client.close();
    await header.evaluate(`document.querySelector('.tab-item.active .tab-close-btn')?.click()`);
    const closeDeadline = Date.now() + 3000;
    while (Date.now() < closeDeadline) {
      const activeTitle = await header.evaluate(
        `document.querySelector('.tab-item.active .tab-title')?.textContent || ''`
      );
      if (!activeTitle.startsWith('Settings')) break;
      await new Promise((resolve) => setTimeout(resolve, 10));
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }

  header.close();
  console.log(JSON.stringify(runs, null, 2));
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
