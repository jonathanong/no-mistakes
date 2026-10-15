const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
test(
  "compiled async CJS and ESM preserve mutable receiver root acquisitions",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/query-reached-per-item/provenance-fixture",
    );
    const config = join(root, "mutable-receivers.yml");
    for (const [name, expected] of [
      ["mutable-let-guarded", 0],
      ["mutable-const-guarded", 0],
      ["mutable-let-direct", 0],
      ["mutable-let-array", 0],
      ["mutable-let-detached", 1],
      ["mutable-let-sibling", 1],
      ["mutable-let-rebound", 0],
      ["mutable-let-rhs-rebound", 0],
      ["mutable-let-rhs-named", 0],
      ["mutable-let-shadowed", 1],
      ["mutable-let-cycle", 0],
      ["mutable-let-copied", 1],
      ["mutable-let-rebound-detaches-old", 1],
      ["mutable-let-self-copy", 0],
      ["mutable-let-conditional-root", 0],
      ["mutable-let-logical-root", 0],
      ["mutable-let-lhs-acquisition", 0],
      ["mutable-let-foreign-root", 0],
    ]) {
      const options = {
        root,
        config,
        kind: "valkey",
        entry: `src/nested-receiver-containers/${name}.mts`,
      };
      const pending = cjs.effects(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(await esm.effects(options), report, name);
      assert.equal(report.callSites.length, expected, name);
      assert.ok(
        report.callSites.every((site) => site.callee === "primary.get"),
        name,
      );
    }
    const report = await cjs.check({ root, config });
    assert.deepEqual(await esm.check({ root, config }), report);
    const findings = report.rules.filter((finding) => finding.rule === "query-reached-per-item");
    assert.equal(findings.length, 1);
    assert.ok(findings[0].file.endsWith("mutable-rule-sibling.mts"));
  },
);
