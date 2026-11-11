const assert = require("node:assert/strict");
const { cp, mkdtemp, readFile, rm } = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

for (const scenario of ["child-read-error", "live-group", "receipt-write-error"]) {
  test(
    `browser diagnostics retain parent evidence after ${scenario}`,
    { timeout: 10000 },
    async () => {
      const { recordBrowserChildDiagnostics } =
        await import("../../fixtures/test-timeout-policy/browser-child-diagnostics.mjs");
      const directory = await mkdtemp(join(tmpdir(), "browser-diagnostic-errors-"));
      await cp(join(__dirname, "../../fixtures/test-timeout-policy/diagnostic-errors"), directory, {
        recursive: true,
      });
      const receipts =
        scenario === "receipt-write-error"
          ? join(directory, "lifecycle.jsonl")
          : join(directory, "receipts.jsonl");
      const previous = process.env.SDK_CHILD_RECEIPT_FILE;
      const previousWarning = console.warn;
      const warnings = [];
      process.env.SDK_CHILD_RECEIPT_FILE = receipts;
      console.warn = (message) => warnings.push(message);
      const failure = new Error("owned group remains live");
      const outcome = {
        result: {
          code: scenario === "live-group" ? null : 17,
          signal: scenario === "live-group" ? "SIGKILL" : null,
        },
        closed: true,
        drained: scenario !== "live-group",
        forcedCleanup: scenario === "live-group",
        errors: scenario === "live-group" ? [failure] : [],
        output: "parent captured compiler output",
        lifecycle: {
          elapsedMs: 10,
          events: [{ phase: "close", elapsedMs: 10 }],
          streams: {},
          diagnosticErrors: [],
        },
      };
      try {
        await recordBrowserChildDiagnostics(outcome, {
          child: { pid: 424242 },
          directory,
          source: scenario,
          environment: {},
          require,
        });
        const artifact = `${receipts}-artifacts/424242`;
        assert.equal(await readFile(join(artifact, "output.log"), "utf8"), outcome.output);
        assert.equal(
          JSON.parse((await readFile(join(artifact, "lifecycle.jsonl"), "utf8")).trim()).phase,
          "close",
        );
        if (scenario !== "receipt-write-error") {
          const receipt = JSON.parse((await readFile(receipts, "utf8")).trim());
          assert.equal(receipt.code, outcome.result.code);
          assert.equal(receipt.drained, outcome.drained);
        }
        if (scenario === "live-group") {
          assert.equal(
            warnings.some((line) => line.includes("child lifecycle unavailable")),
            false,
          );
          assert.equal(outcome.errors[0], failure);
        } else {
          assert.ok(
            warnings.some(
              (line) => line.includes("child lifecycle unavailable") && line.includes("EISDIR"),
            ),
          );
          assert.deepEqual(outcome.errors, []);
          assert.equal(outcome.result.code, 17);
        }
        if (scenario === "receipt-write-error")
          assert.ok(
            warnings.some(
              (line) => line.includes("receipt unavailable") && line.includes("EISDIR"),
            ),
          );
      } finally {
        console.warn = previousWarning;
        if (previous === undefined) delete process.env.SDK_CHILD_RECEIPT_FILE;
        else process.env.SDK_CHILD_RECEIPT_FILE = previous;
        await rm(directory, { recursive: true, force: true });
      }
    },
  );
}
