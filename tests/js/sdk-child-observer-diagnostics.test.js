const assert = require("node:assert/strict");
const { spawn } = require("node:child_process");
const { mkdtemp, readFile, rm } = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

for (const mode of ["pass", "reject", "startup-stall", "shutdown-stall"]) {
  test(
    `SDK lifecycle receipts preserve ${mode} outcome and owned cleanup`,
    { timeout: 10000 },
    async () => {
      if (process.platform !== "linux") return;
      const { captureOwnedChild, observeSdkChild } =
        await import("../../fixtures/test-timeout-policy/owned-sdk-child.mjs");
      const { recordBrowserChildDiagnostics } =
        await import("../../fixtures/test-timeout-policy/browser-child-diagnostics.mjs");
      const directory = await mkdtemp(join(tmpdir(), "sdk-child-diagnostics-"));
      const receipts = join(directory, "receipts.jsonl");
      const priorReceipt = process.env.SDK_CHILD_RECEIPT_FILE;
      process.env.SDK_CHILD_RECEIPT_FILE = receipts;
      const child = spawn(
        process.execPath,
        [join(__dirname, "../../fixtures/test-timeout-policy/child-lifecycle/child.mjs"), mode],
        {
          cwd: directory,
          detached: true,
          stdio: ["ignore", "pipe", "pipe"],
          env: {
            ...process.env,
            BROWSER_LIFECYCLE_RECEIPT_FILE: join(directory, "lifecycle.jsonl"),
          },
        },
      );
      let terminate;
      try {
        const outcome = await observeSdkChild(child, {
          watchdogMs: 5000,
          capture: (value) => (terminate = captureOwnedChild(value)),
        });
        const stalled = mode.endsWith("-stall");
        assert.equal(outcome.closed, true);
        assert.equal(outcome.drained, true);
        assert.deepEqual(outcome.errors, []);
        assert.equal(outcome.forcedCleanup, stalled);
        assert.deepEqual(
          outcome.result,
          stalled
            ? { code: null, signal: "SIGKILL" }
            : { code: mode === "reject" ? 1 : 0, signal: null },
        );
        const phases = outcome.lifecycle.events.map((event) => event.phase);
        for (const phase of [
          "observe-start",
          "spawn",
          "stderr-first",
          "stderr-last",
          "exit",
          "close",
          "group-drain-start",
          "group-drain-end",
        ])
          assert.ok(phases.includes(phase), phase);
        assert.equal(phases.includes("watchdog"), stalled);
        assert.ok(outcome.lifecycle.streams.stderr.bytes > 0);
        if (mode !== "startup-stall") assert.match(outcome.output, /FIXTURE_BODY_COMPLETED/);
        await recordBrowserChildDiagnostics(outcome, {
          child,
          directory,
          source: mode,
          environment: {},
          require,
        });
        const receipt = JSON.parse((await readFile(receipts, "utf8")).trim());
        assert.equal(receipt.closed, true);
        assert.equal(receipt.drained, true);
        const artifact = await readFile(
          `${receipts}-artifacts/${child.pid}/lifecycle.jsonl`,
          "utf8",
        );
        const events = artifact
          .trim()
          .split("\n")
          .map((line) => JSON.parse(line));
        assert.equal(
          events.some((event) => event.phase === "onTestRunEnd:complete"),
          mode !== "startup-stall",
        );
        assert.equal(
          events.some((event) => event.phase === "watchdog"),
          stalled,
        );
      } finally {
        if (priorReceipt === undefined) delete process.env.SDK_CHILD_RECEIPT_FILE;
        else process.env.SDK_CHILD_RECEIPT_FILE = priorReceipt;
        terminate?.();
        child.stdout.destroy();
        child.stderr.destroy();
        await rm(directory, { recursive: true, force: true });
      }
    },
  );
}
