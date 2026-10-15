const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const root = join(__dirname, "../../../test-cases/rules/query-reached-per-item/provenance-fixture");
const config = join(root, "mutable-receivers.yml");
// Each native CJS/ESM pair has its own existing runner deadline on slower CI hosts.
for (const [name, expected] of [
  ["mutable-affected-cross-scope", 2],
  ["terminal-receiver-history", 0],
  ["terminal-import-with-unrelated-root", 0],
  ["mutable-let-pattern-static-var-conditional", 0],
  ["mutable-let-pattern-replaced-earlier-lhs", 1],
  ["mutable-let-pattern-replaced-rhs-rebind", 1],
  ["mutable-let-pattern-static-local", 1],
  ["mutable-let-pattern-static-fields", 1],
  ["mutable-let-pattern-static-rebind", 1],
  ["mutable-let-pattern-ternary-rebind", 0],
  ["mutable-let-pattern-final-foreign", 0],
  ["mutable-let-pattern-conditional-whole", 1],
  ["mutable-let-pattern-conditional-final", 0],
  ["mutable-let-pattern-final-root", 1],
  ["mutable-let-pattern-foreign-catalog", 1],
  ["mutable-let-pattern-logical-rebind", 0],
  ["mutable-let-pattern-foreign-rebind", 0],
  ["mutable-let-pattern-shorthand", 1],
  ["mutable-let-pattern-rhs-rebind", 1],
  ["mutable-let-pattern-lhs-rebind", 1],
  ["mutable-let-pattern-nested-before-rhs", 0],
  ["mutable-let-pattern-conditional-rebind", 0],
  ["mutable-let-pattern-self-copy", 0],
  ["mutable-let-pattern-twice", 1],
  ["mutable-let-pattern-before", 0],
  ["mutable-let-wrapped-unrelated", 1],
  ["mutable-let-pattern-wrapped", 1],
  ["mutable-let-pattern-lhs", 1],
  ["mutable-let-pattern-rhs", 0],
  ["mutable-let-pattern-default", 1],
  ["mutable-let-wrapped-as", 1],
  ["mutable-let-wrapped-non-null", 1],
  ["mutable-let-wrapped-assertion", 1],
  ["mutable-let-wrapped-satisfies", 1],
  ["mutable-let-destructured-array", 1],
  ["mutable-let-destructured-object", 1],
  ["mutable-let-destructured-default", 1],
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
  test(
    `compiled async CJS and ESM preserve mutable receiver root acquisitions: ${name}`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
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
      if (name === "mutable-affected-cross-scope") {
        // The copied method and untouched receiver remain; the changed read does not.
        assert.deepEqual(
          report.callSites.map(({ line, callee }) => [line, callee]).sort((a, b) => a[0] - b[0]),
          [
            [9, "primary.get"],
            [12, "replica.get"],
          ],
        );
      } else {
        assert.ok(
          report.callSites.every((site) => site.callee === "primary.get"),
          name,
        );
      }
    },
  );
}
for (const name of [
  "mutable-let-pattern-static-capture",
  "mutable-let-pattern-static-comma-capture",
  "mutable-let-pattern-static-self-wrapper",
  "mutable-let-pattern-replaced-getter-rhs",
  "mutable-let-pattern-static-nested",
  "mutable-let-pattern-class-context",
  "mutable-let-pattern-cold-class",
  "mutable-let-pattern-shadowed-class",
]) {
  test(
    `compiled async CJS and ESM preserve mutable receiver root acquisitions: ${name}`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const options = {
        root,
        config,
        kind: "valkey",
        entry: `src/nested-receiver-containers/${name}.mts`,
      };
      const report = await cjs.effects(options);
      assert.deepEqual(await esm.effects(options), report, name);
      assert.deepEqual(
        report.callSites.map((site) => site.callee),
        ["replica.get"],
        name,
      );
    },
  );
}
for (const name of [
  "mutable-let-pattern-replaced-later-lhs",
  "mutable-let-pattern-replaced-carrier",
]) {
  test(
    `compiled async CJS and ESM preserve mutable receiver root acquisitions: ${name}`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const detached = {
        root,
        config,
        kind: "valkey",
        entry: `src/nested-receiver-containers/${name}.mts`,
      };
      const detachedReport = await cjs.effects(detached);
      assert.deepEqual(await esm.effects(detached), detachedReport);
      assert.deepEqual(
        detachedReport.callSites.map((site) => site.callee).sort(),
        ["primary.get", "replica.get"],
        name,
      );
    },
  );
}
test(
  "compiled async CJS and ESM preserve mutable receiver root acquisitions: check parity",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const report = await cjs.check({ root, config });
    assert.deepEqual(await esm.check({ root, config }), report);
    const findings = report.rules.filter((finding) => finding.rule === "query-reached-per-item");
    assert.equal(findings.length, 1);
    assert.ok(findings[0].file.endsWith("mutable-rule-sibling.mts"));
  },
);
