const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const root = join(__dirname, "../../../test-cases/rules/query-reached-per-item/provenance-fixture");
const config = join(root, "mutable-receivers.yml");
const effectsRequests = [];
let prepared;

function effectsEntry(entry) {
  effectsRequests.push({ type: "effects", id: entry, kind: "valkey", entry });
  return entry;
}

async function analyzeEffects(api) {
  const pending = api.analyzeProject({ root, config, reports: effectsRequests });
  assert.ok(pending instanceof Promise);
  const { reports } = await pending;
  assert.equal(reports.length, effectsRequests.length);
  return new Map(reports.map(({ id, result }) => [id, result]));
}

// Retain one request per facade in this shard. Cases only read the results.
async function preparedEffects(entry) {
  prepared ??= Promise.all([
    analyzeEffects(require("../index.js")),
    import("../index.mjs").then(analyzeEffects),
  ]);
  const [cjs, esm] = await prepared;
  return [cjs.get(entry), esm.get(entry)];
}

for (const [name, expected] of [
  ["mutable-affected-cross-scope", 2],
  ["mutable-affected-alias-overflow", 2],
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
  const extension = name === "mutable-let-wrapped-assertion" ? "ts" : "mts";
  const entry = effectsEntry(`src/nested-receiver-containers/${name}.${extension}`);
  test(
    `compiled async CJS and ESM preserve mutable receiver root acquisitions: ${name}`,
    { skip: !compiled },
    async () => {
      const [report, esm] = await preparedEffects(entry);
      assert.deepEqual(esm, report, name);
      assert.equal(report.callSites.length, expected, name);
      if (name === "mutable-affected-cross-scope" || name === "mutable-affected-alias-overflow") {
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

for (const [entry, expected] of [
  ["module-writer-rebind-before.mts", ["primary.get", "primary.get"]],
  ["module-writer-write-before-rebind.mts", ["replica.get"]],
  ["module-writer-declaration-after-rebind.mts", []],
  ["module-writer-finishing-same-owner-chain.mts", []],
]) {
  const source = effectsEntry(`src/${entry}`);
  test(
    `compiled async CJS and ESM preserve module-owned deferred receiver writes: ${entry}`,
    { skip: !compiled },
    async () => {
      const [report, esm] = await preparedEffects(source);
      assert.deepEqual(esm, report, entry);
      assert.deepEqual(
        report.callSites.map(({ callee }) => callee),
        expected,
        entry,
      );
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
  const entry = effectsEntry(`src/nested-receiver-containers/${name}.mts`);
  test(
    `compiled async CJS and ESM preserve mutable receiver root acquisitions: ${name}`,
    { skip: !compiled },
    async () => {
      const [report, esm] = await preparedEffects(entry);
      assert.deepEqual(esm, report, name);
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
  const entry = effectsEntry(`src/nested-receiver-containers/${name}.mts`);
  test(
    `compiled async CJS and ESM preserve mutable receiver root acquisitions: ${name}`,
    { skip: !compiled },
    async () => {
      const [detachedReport, esm] = await preparedEffects(entry);
      assert.deepEqual(esm, detachedReport);
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

for (const [name, expected] of [
  ["controls", ["primary.get", "replica.get"]],
  ["capture-before", ["replica.get"]],
  ["capture-after", ["primary.get"]],
  ["uncalled", ["primary.get", "replica.get"]],
  ["escaped", []],
  ["cycle", []],
  ["conditional", []],
  ["reset", ["replica.get"]],
  ["reordered-rebind", ["replica.get"]],
  ["reordered-copy", ["replica.get"]],
  ["eager-class", ["replica.get"]],
  ["argument-rebind", []],
]) {
  const entry = effectsEntry(`src/module-writer-${name}.mts`);
  test(
    `compiled async CJS and ESM preserve module provider guard controls: ${name}`,
    { skip: !compiled },
    async () => {
      const [report, esm] = await preparedEffects(entry);
      assert.deepEqual(esm, report, name);
      assert.deepEqual(
        report.callSites.map((site) => site.callee),
        expected,
        name,
      );
    },
  );
}

for (const name of [
  "providers-16-reads-16",
  "providers-256-reads-16",
  "providers-256-reads-128",
  "providers-256-reads-128-diamond",
]) {
  test(
    `compiled async CJS and ESM preserve bounded module provider projections: ${name}`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const fixture = join(
        __dirname,
        "../../../test-cases/codebase-analysis/module-receiver-provider-scale/fixture",
      );
      const options = {
        root: join(fixture, name),
        config: join(fixture, "effects.yml"),
        kind: "receiver",
        entry: "src/entry.mts",
      };
      const pending = cjs.effects(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(await esm.effects(options), report, name);
      assert.deepEqual(report.callSites, [], name);
    },
  );
}

test(
  "compiled mutable receiver standalone effects match prepared reports",
  { skip: !compiled },
  async () => {
    const entry = "src/nested-receiver-containers/mutable-affected-cross-scope.mts";
    const [cjs, esm] = await preparedEffects(entry);
    const options = { root, config, kind: "valkey", entry };
    const cjsPending = require("../index.js").effects(options);
    const esmApi = await import("../index.mjs");
    const esmPending = esmApi.effects(options);
    assert.ok(cjsPending instanceof Promise);
    assert.ok(esmPending instanceof Promise);
    assert.deepEqual(await cjsPending, cjs);
    assert.deepEqual(await esmPending, esm);
  },
);
