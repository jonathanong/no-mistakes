const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "severity: advisory findings land in advisories through check() and analyzeProject()",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const root = join(__dirname, "../../../test-cases/config-v2/rule-severity-advisory/fixture");
    const report = await api.check({ root });
    assert.deepEqual(report.rules, []);
    assert.deepEqual(
      report.advisories.map(({ file, rule }) => [file, rule]),
      [["placeholder.ts", "no-empty-or-comments-only-files"]],
    );
    const batch = await api.analyzeProject({ root, reports: [{ type: "check" }] });
    assert.deepEqual(batch.reports[0].result, report);
  },
);
