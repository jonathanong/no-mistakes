const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const root = join(__dirname, "../../../test-cases/rules/query-reached-per-item/provenance-fixture");
const config = join(root, "mutable-receivers.yml");
const replacementConfig = join(root, "assignment-receivers.yml");
// Each physical fixture has its own normal CI deadline, including both facades.
for (const [name, expected] of [
  ["retained", ["replica.get"]],
  ["preindex-rebound-parent", ["primary.get"]],
  ["rebound-parent-sibling", ["primary.get"]],
  ["rebound-parent-scope", ["primary.get"]],
  ["rebound-parent-conditional", []],
  ["rebound-parent-unknown", []],
  ["late-alias", ["replica.get"]],
  ["member-source", []],
  ["mixed-prefix", []],
  ["pattern-member", ["replica.get"]],
  ["pattern-shorthand-default", ["primary.get"]],
  ["pattern-rest-property", ["primary.get"]],
  ["pattern-named", ["primary.get"]],
  ["pattern-odd-dynamic", []],
  ["pattern-empty-dynamic", ["primary.get"]],
  ["pattern", ["primary.get"]],
  ["pattern-order", ["primary.get"]],
  ["copied", ["primary.get"]],
  ["restored", ["replica.get"]],
  ["snapshot", ["replica.get"]],
  ["direct", ["primary.get"]],
  ["unknown", []],
  ["conditional", []],
  ["unknown-ancestor", []],
  ["saved-choice", []],
  ["saved-choice-reverse", []],
  ["mutable-member", ["replica.get"]],
  ["mutable-previous", ["replica.get"]],
  ["mutable-rebound", ["primary.get"]],
  ["preindex-fresh", ["primary.get"]],
  ["postindex-fresh", ["primary.get"]],
  ["preindex-root-rebound", ["primary.get"]],
  ["preindex-unknown", []],
  ["preindex-conditional", []],
  ["preindex-deep", ["primary.get"]],
  ["preindex-mutable", ["primary.get"]],
]) {
  test(
    `compiled async CJS and ESM preserve replacement RHS aliases: ${name}`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const options = {
        root,
        config,
        kind: "valkey",
        entry: `src/nested-receiver-containers/assignment-rhs-${name}.mts`,
      };
      const cjsPending = cjs.effects(options);
      assert.ok(cjsPending instanceof Promise);
      const report = await cjsPending;
      const esmPending = esm.effects(options);
      assert.ok(esmPending instanceof Promise);
      assert.deepEqual(await esmPending, report, name);
      assert.deepEqual(report.callSites.map((site) => site.callee).sort(), expected, name);
    },
  );
}

test(
  "compiled async CJS and ESM preserve replacement RHS aliases: prepared report parity",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const options = {
      root,
      config: replacementConfig,
      kind: "valkey",
      entry: "src/nested-receiver-containers/assignment-rhs-retained.mts",
    };
    const direct = await cjs.effects(options);
    const request = {
      root,
      config: replacementConfig,
      reports: [
        { type: "effects", id: "direct", kind: options.kind, entry: options.entry },
        {
          type: "effects",
          id: "saved",
          kind: options.kind,
          entry: "src/nested-receiver-containers/assignment-rhs-snapshot.mts",
        },
        { type: "check", id: "checks" },
      ],
    };
    const pending = cjs.analyzeProject(request);
    assert.ok(pending instanceof Promise);
    const batch = await pending;
    assert.deepEqual(batch.reports.find((report) => report.id === "direct").result, direct);
    assert.deepEqual(
      batch.reports
        .find((report) => report.id === "saved")
        .result.callSites.map((site) => site.callee),
      ["replica.get"],
    );
    const findings = batch.reports.find((report) => report.id === "checks").result.rules;
    assert.equal(findings.length, 1);
    assert.ok(findings[0].file.endsWith("assignment-rhs-rule-sibling.mts"));
    const esmPending = esm.analyzeProject(request);
    assert.ok(esmPending instanceof Promise);
    assert.deepEqual(await esmPending, batch);
  },
);

test(
  "compiled async CJS and ESM preserve replacement RHS aliases: check parity",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const pending = cjs.check({ root, config: replacementConfig });
    assert.ok(pending instanceof Promise);
    const report = await pending;
    const esmPending = esm.check({ root, config: replacementConfig });
    assert.ok(esmPending instanceof Promise);
    assert.deepEqual(await esmPending, report);
    assert.equal(report.rules.length, 1);
    assert.ok(report.rules[0].file.endsWith("assignment-rhs-rule-sibling.mts"));
  },
);
