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
    for (const [file, resolved] of [
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
    ]) {
      const options = { root, kind: "valkey", entry: `src/nested-receiver-containers/${file}.mts` };
      const pending = cjs.effects(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(await esm.effects(options), report, file);
      assert.equal(
        report.callSites.some((site) => site.caller === "scan" && site.callee === "primary.get"),
        resolved,
        file,
      );
    }
  },
);
