const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
test(
  "compiled effects preserve decorator application receiver barriers",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/query-reached-per-item/provenance-fixture",
    );
    const cases = [
      ["global-decorator-application", false],
      ["global-decorator-late-key", true],
      ["global-decorator-anonymous-late-key", true],
      ["global-decorator-nested-class", false],
      ["global-decorator-static-block", false],
      ["global-decorator-static-field", false],
      ["global-decorator-accessor", false],
      ["global-decorator-anonymous", false],
      ["global-decorator-anonymous-late-member", false],
      ["global-decorator-member-static", false],
      ["global-decorator-factory", false],
      ["global-decorator-before-capture", true],
      ["global-decorator-class-static", true],
      ["global-decorator-computed-key", true],
      ["global-decorator-copied-snapshot", true],
      ["global-decorator-deferred", true],
    ];
    // Share one analysis per facade while keeping every decorator report assertion.
    const reports = cases.map(([file]) => ({
      type: "effects",
      kind: "valkey",
      entry: `src/nested-receiver-containers/${file}.mts`,
    }));
    const pending = cjs.analyzeProject({ root, reports });
    assert.ok(pending instanceof Promise);
    const result = await pending;
    const esmPending = esm.analyzeProject({ root, reports });
    assert.ok(esmPending instanceof Promise);
    assert.deepEqual(await esmPending, result);
    for (const [index, [file, resolved]] of cases.entries()) {
      const report = result.reports[index].result;
      assert.equal(
        report.callSites.some((site) => site.caller === "scan" && site.callee === "primary.get"),
        resolved,
        file,
      );
    }
  },
);
