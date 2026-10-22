const assert = require("node:assert/strict");
const { fork } = require("node:child_process");
const { once } = require("node:events");
const { mkdtemp, rm } = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const { setTimeout: delay } = require("node:timers/promises");
const { Worker } = require("node:worker_threads");
const test = globalThis.test || require("node:test").test;

const repositoryRoot = join(__dirname, "..", "..", "..");
const addonPath = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const compiledAddonPath = addonPath && addonPath.endsWith(".node") ? addonPath : undefined;

for (const workers of [1, 3]) {
  test(
    `compiled N-API live progress polling works with all ${workers} libuv workers occupied`,
    { skip: !compiledAddonPath || process.platform !== "linux", timeout: 10_000 },
    async () => {
      const directory = await mkdtemp(join(tmpdir(), "no-mistakes-live-progress-"));
      const child = fork(
        join(repositoryRoot, "fixtures", "napi", "diagnostics-progress", "poll.js"),
        [
          directory,
          join(
            repositoryRoot,
            "test-cases",
            "codebase-analysis",
            "check-configured-only",
            "fixture",
          ),
        ],
        {
          env: {
            ...process.env,
            NO_MISTAKES_TEST_NAPI_ADDON_PATH: addonPath,
            UV_THREADPOOL_SIZE: String(workers),
          },
          stdio: ["ignore", "pipe", "pipe", "ipc"],
        },
      );
      let stderr = "";
      child.stderr.on("data", (chunk) => {
        stderr += chunk;
      });
      try {
        const [code, signal] = await once(child, "exit");
        assert.equal(signal, null, stderr);
        assert.equal(code, 0, stderr);
        assert.doesNotMatch(stderr, /\[progress\]|\[timing\]/);
      } finally {
        child.kill();
        await rm(directory, { recursive: true, force: true });
      }
    },
  );
}

test(
  "compiled polling rejects malformed requests and overload while every call remains a promise",
  { skip: !compiledAddonPath, timeout: 10_000 },
  async () => {
    const native = require(addonPath);
    for (const input of [Buffer.from([0xff]), Buffer.from("{"), Buffer.from('{"id":123}')]) {
      const pending = native.diagnosticsSnapshotJson(input);
      assert.equal(typeof pending.then, "function");
      await assert.rejects(pending);
    }
    // Parsing this raw request occupies the dedicated worker while a burst
    // fills its bounded queue. Rejections must settle, never strand promises.
    const heavy = Buffer.from(`{"id":"missing","padding":[${"0,".repeat(1_000_000)}0]}`);
    const pending = [native.diagnosticsSnapshotJson(heavy)];
    for (let index = 0; index < 256; index++) {
      const result = native.diagnosticsSnapshotJson(Buffer.from('{"id":"missing"}'));
      assert.equal(typeof result.then, "function");
      pending.push(result);
    }
    const outcomes = await Promise.allSettled(pending);
    assert.ok(
      outcomes.some(
        (result) => result.status === "rejected" && /queue is full/.test(result.reason.message),
      ),
    );
    assert.equal(
      JSON.parse(await native.diagnosticsSnapshotJson(Buffer.from('{"id":"missing"}'))),
      null,
    );
  },
);

test(
  "compiled polling owns request bytes and survives terminating a Worker with queued promises",
  { skip: !compiledAddonPath, timeout: 10_000 },
  async () => {
    const native = require(addonPath);
    const heavy = Buffer.from(`{"id":"missing","padding":[${"0,".repeat(1_000_000)}0]}`);
    const first = native.diagnosticsSnapshotJson(heavy).catch(() => {});
    const input = Buffer.from('{"id":"missing"}');
    const poll = native.diagnosticsSnapshotJson(input);
    input.fill(0xff);
    assert.equal(JSON.parse(await poll), null);
    await first;
    const worker = new Worker(
      join(repositoryRoot, "fixtures", "napi", "diagnostics-progress", "terminate.js"),
      {
        workerData: { addonPath },
      },
    );
    try {
      const [message] = await once(worker, "message");
      assert.equal(message, "queued");
      await worker.terminate();
      // Closed-isolate jobs may still occupy the bounded queue briefly. Retry
      // overload until an unrelated live isolate receives its own result.
      let recovered = false;
      const end = Date.now() + 2000;
      while (Date.now() < end) {
        try {
          assert.equal(
            JSON.parse(await native.diagnosticsSnapshotJson(Buffer.from('{"id":"missing"}'))),
            null,
          );
          recovered = true;
          break;
        } catch (error) {
          assert.match(error.message, /queue is full/);
          await delay(5);
        }
      }
      assert.equal(recovered, true);
    } finally {
      await worker.terminate();
    }
  },
);
