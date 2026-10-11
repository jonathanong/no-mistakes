const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addonPath = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const fixtureRoot = join(__dirname, "..", "..", "..", "test-cases", "tests-audit");

test(
  "compiled audit API preserves async evidence and batch parity",
  { skip: !addonPath?.endsWith(".node"), timeout: 20_000 },
  async () => {
    const api = require("../index.js");
    const options = {
      plan: join(fixtureRoot, "plan.json"),
      observations: join(fixtureRoot, "observations.json"),
    };
    const pending = api.testsAudit(options);
    assert.equal(typeof pending.then, "function");
    const direct = await pending;
    assert.deepEqual(direct.missedObservedTests, [
      {
        testFile: "tests/missed.test.mts",
        matchedFiles: [],
        matchedSymbols: [{ file: "src/b.mts", symbol: "changed" }],
      },
    ]);
    assert.deepEqual(direct.selectedWithIncompleteTraces, ["tests/incomplete.test.mts"]);
    const batched = await api.analyzeProject({
      root: fixtureRoot,
      reports: [{ type: "testsAudit", ...options }],
    });
    assert.deepEqual(batched.reports[0].result, direct);
    assert.deepEqual(
      await api.testsAudit({
        planJson: JSON.parse(readFileSync(options.plan, "utf8")),
        observationsJson: readFileSync(options.observations, "utf8"),
      }),
      direct,
    );
    await assert.rejects(
      api.testsAudit({ ...options, plan: join(fixtureRoot, "mismatched-plan.json") }),
      /provenance mismatch/,
    );
  },
);
