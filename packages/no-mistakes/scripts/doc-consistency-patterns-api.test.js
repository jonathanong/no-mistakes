const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled document regex policies preserve async checks and suppression parity",
  { skip: !compiled },
  async () => {
    const root = join(__dirname, "../../../test-cases/rules/doc-consistency/fixture/patterns");
    const options = { root, config: join(root, ".no-mistakes.yml") };
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const reports = [];
    for (const api of [cjs, esm]) {
      const pending = api.check(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(
        report.rules.map(({ file, line }) => [file, line]),
        [
          ["README.md", 4],
          ["docs/guide.md", 3],
          ["docs/guide.md", 4],
          ["docs/guide.md", 6],
          ["docs/suppressed.md", 4],
        ],
      );
      assert.equal(
        report.rules[0].message,
        "Keep live pins in tools/pins.json; link that owner file.",
      );
      const prepared = api.analyzeProject({ ...options, reports: [{ type: "check" }] });
      assert.ok(prepared instanceof Promise);
      assert.deepEqual((await prepared).reports[0].result, report);
      reports.push(report);
    }
    assert.deepEqual(reports[0], reports[1]);
  },
);
