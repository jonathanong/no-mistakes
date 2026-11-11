const assert = require("node:assert/strict");
const { readFile, mkdtemp, rm } = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

async function observeBrowserProvider(option) {
  return (
    await import("../../fixtures/test-timeout-policy/browser-lifecycle-probe.mjs")
  ).observeBrowserProvider(option);
}
async function probe() {
  return import("../../fixtures/test-timeout-policy/browser-lifecycle-probe.mjs");
}

async function withReceipt(run) {
  const directory = await mkdtemp(join(tmpdir(), "browser-lifecycle-test-"));
  const previous = process.env.BROWSER_LIFECYCLE_RECEIPT_FILE;
  process.env.BROWSER_LIFECYCLE_RECEIPT_FILE = join(directory, "lifecycle.jsonl");
  try {
    return await run(async () => {
      const content = await readFile(process.env.BROWSER_LIFECYCLE_RECEIPT_FILE, "utf8");
      return content
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line));
    });
  } finally {
    if (previous === undefined) delete process.env.BROWSER_LIFECYCLE_RECEIPT_FILE;
    else process.env.BROWSER_LIFECYCLE_RECEIPT_FILE = previous;
    await rm(directory, { recursive: true, force: true });
  }
}

test(
  "provider instrumentation preserves factory and provider identity and arguments",
  { timeout: 10000 },
  async () => {
    await withReceipt(async (records) => {
      const context = { marker: "factory-this" };
      const provider = {
        openPage: async function (sessionId, url, options) {
          assert.equal(this, provider);
          assert.deepEqual(
            [sessionId, url, options.parallel],
            ["opaque-session", "https://private.invalid", 2],
          );
          return "page-result";
        },
        close: function (options) {
          assert.equal(this, provider);
          assert.deepEqual(options, { reason: "done" });
          return "close-result";
        },
      };
      const option = {
        providerFactory: function (project) {
          assert.equal(this, context);
          assert.deepEqual(project, { name: "project" });
          return provider;
        },
      };
      const observed = await observeBrowserProvider(option);
      assert.equal(observed, option);
      assert.equal(option.providerFactory.call(context, { name: "project" }), provider);
      assert.equal(
        await provider.openPage("opaque-session", "https://private.invalid", { parallel: 2 }),
        "page-result",
      );
      assert.equal(provider.close({ reason: "done" }), "close-result");

      const events = await records();
      assert.deepEqual(
        events.map((event) => event.phase),
        [
          "provider.factory:start",
          "provider.factory:complete",
          "provider.openPage:start",
          "provider.openPage:complete",
          "provider.close:start",
          "provider.close:complete",
        ],
      );
      assert.ok(
        events.every((event) => typeof event.at === "string" && Number.isFinite(event.atMs)),
      );
      assert.equal(
        events.some((event) => JSON.stringify(event).includes("opaque-session")),
        false,
      );
      assert.equal(
        events.some((event) => JSON.stringify(event).includes("private.invalid")),
        false,
      );
    });
  },
);

test(
  "provider close completion records the end of a blocked public close",
  { timeout: 10000 },
  async () => {
    await withReceipt(async (records) => {
      let release;
      const closePromise = new Promise((resolve) => {
        release = resolve;
      });
      const provider = {
        openPage() {},
        close() {
          return closePromise;
        },
      };
      const option = { providerFactory: () => provider };
      await observeBrowserProvider(option);
      assert.equal(option.providerFactory(), provider);
      const result = provider.close();
      assert.equal(result, closePromise);
      await Promise.resolve();
      const before = await records();
      assert.equal(before.at(-1).phase, "provider.close:start");
      release();
      await result;
      await Promise.resolve();
      assert.deepEqual(
        (await records()).slice(-2).map((event) => event.phase),
        ["provider.close:start", "provider.close:complete"],
      );
    });
  },
);

test(
  "provider wrapper preserves synchronous throws and observes async rejection",
  { timeout: 10000 },
  async () => {
    await withReceipt(async (records) => {
      const failure = new Error("provider failed");
      const rejected = Promise.reject(failure);
      rejected.catch(() => {});
      const provider = {
        openPage() {
          throw failure;
        },
        close() {
          return rejected;
        },
      };
      const option = { providerFactory: () => provider };
      await observeBrowserProvider(option);
      option.providerFactory();
      assert.throws(
        () => provider.openPage(),
        (error) => error === failure,
      );
      assert.equal(provider.close(), rejected);
      await rejected.catch(() => {});
      await new Promise((resolve) => setImmediate(resolve));
      const events = await records();
      assert.ok(
        events.some(
          (event) => event.phase === "provider.openPage:error" && event.errorName === "Error",
        ),
      );
      assert.ok(
        events.some(
          (event) => event.phase === "provider.close:error" && event.errorName === "Error",
        ),
      );
      assert.equal(
        events.some((event) => JSON.stringify(event).includes("provider failed")),
        false,
      );
    });
  },
);

test(
  "missing reporter hooks still produce lifecycle receipts and Vitest close callback",
  { timeout: 10000 },
  async () => {
    await withReceipt(async (records) => {
      const { installBrowserLifecycleReceipts } = await probe();
      const callbacks = [];
      const reporter = {};
      assert.equal(installBrowserLifecycleReceipts(reporter), reporter);
      reporter.onInit({
        onClose(callback) {
          callbacks.push(callback);
        },
      });
      reporter.onTestRunStart([]);
      reporter.onTestCaseResult({
        name: "passed test",
        result() {
          return { state: "passed" };
        },
      });
      reporter.onTestRunEnd([], [], "passed");
      callbacks[0]();
      const events = await records();
      for (const phase of [
        "onTestRunStart:start",
        "onTestRunStart:complete",
        "onTestCaseResult:start",
        "onTestCaseResult:complete",
        "onTestRunEnd:start",
        "onTestRunEnd:complete",
        "vitest-close-callback",
      ])
        assert.ok(
          events.some((event) => event.phase === phase),
          phase,
        );
      assert.ok(
        events.some(
          (event) => event.phase === "onTestCaseResult:start" && event.testState === "passed",
        ),
      );
    });
  },
);

test(
  "existing reporter hook return, rejection, throw, this, and arguments are preserved",
  { timeout: 10000 },
  async () => {
    await withReceipt(async (records) => {
      const { installBrowserLifecycleReceipts } = await probe();
      const context = { marker: "reporter-this" };
      const input = { marker: "reporter-arg" };
      const failure = new Error("private callback details");
      const rejection = Promise.reject(failure);
      rejection.catch(() => {});
      const reporter = {
        onTestRunStart(value) {
          assert.equal(this, context);
          assert.equal(value, input);
          return rejection;
        },
        onTestRunEnd() {
          throw failure;
        },
      };
      installBrowserLifecycleReceipts(reporter);
      assert.equal(reporter.onTestRunStart.call(context, input), rejection);
      assert.throws(
        () => reporter.onTestRunEnd(),
        (error) => error === failure,
      );
      await rejection.catch(() => {});
      await new Promise((resolve) => setImmediate(resolve));
      const events = await records();
      assert.ok(
        events.some(
          (event) => event.phase === "onTestRunStart:error" && event.errorName === "Error",
        ),
      );
      assert.ok(
        events.some((event) => event.phase === "onTestRunEnd:error" && event.errorName === "Error"),
      );
      assert.equal(
        events.some((event) => JSON.stringify(event).includes("private callback details")),
        false,
      );
    });
  },
);
