const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { join } = require("node:path");
const { readFileSync } = require("node:fs");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
test(
  "compiled CJS and ESM preserve single inline method acquisitions",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/inline-member-precision/fixture",
    );
    const cases = [
      ["unique-helper-repeated", 0],
      ["unique-helper-snapshot", 1],
      ["unique-helper-setup", 1],
      ["inline-snapshot-owned", 1],
      ["default-snapshot-owned", 1],
      ["default-object-snapshot-owned", 1],
      ["default-snapshot-write-first", 0],
      ["default-snapshot-deferred-arrow", 0],
      ["default-snapshot-deferred-function", 0],
      ["default-snapshot-deferred-method", 0],
      ["default-snapshot-nested", 0],
      ["default-snapshot-loop", 0],
      ["default-snapshot-field", 0],
      ["default-snapshot-accessor", 0],
      ["default-snapshot-async", 0],
      ["default-snapshot-reentrant", 0],
      ["default-computed-arrow", 0],
      ["default-computed-function", 0],
      ["default-computed-method", 0],
      ["default-computed-class", 0],
      ["default-computed-parameter", 0],
      ["default-computed-key", 1],
      ["default-computed-class-key", 1],
      ["default-computed-restored", 1],
      ["module-computed-arrow", 0],
      ["alias-inline-snapshot-owned", 1],
      ["inline-snapshot-owned-write-first", 0],
      ["inline-snapshot-repeated-for", 0],
      ["inline-snapshot-repeated-header", 0],
      ["inline-snapshot-repeated-while", 0],
      ["inline-snapshot-repeated-do", 0],
      ["inline-snapshot-repeated-function", 0],
      ["inline-snapshot-repeated-field", 0],
      ["inline-snapshot-repeated-nested-field", 0],
      ["inline-snapshot-repeated-accessor", 0],
    ];
    cases.push(
      ["loop-setup-annex.cjs", 0],
      ["loop-setup-async.mts", 0],
      ["loop-setup-for-declaration.mts", 1],
      ["loop-setup-for-expression.mts", 1],
      ["loop-setup-for-update.mts", 0],
      ["loop-setup-function.mts", 0],
      ["loop-setup-in-body.mts", 0],
      ["loop-setup-in-default.mts", 0],
      ["loop-setup-in-rhs.mts", 1],
      ["loop-setup-in-target.mts", 0],
      ["loop-setup-nested-for.mts", 0],
      ["loop-setup-nested-in.mts", 0],
      ["loop-setup-nested-of.mts", 0],
      ["loop-setup-of-body.mts", 0],
      ["loop-setup-of-default.mts", 0],
      ["loop-setup-of-rhs.mts", 1],
      ["loop-setup-of-target.mts", 0],
    );
    const reports = cases.map(([name]) => ({
      type: "effects",
      kind: "cache",
      entry: `src/${name.includes(".") ? name : `${name}.mts`}`,
    }));
    const pending = cjs.analyzeProject({ root, reports });
    assert.ok(pending instanceof Promise);
    const result = await pending;
    assert.deepEqual(await esm.analyzeProject({ root, reports }), result);
    for (const [index, [name, expected]] of cases.entries()) {
      const sites = result.reports[index].result.callSites;
      assert.equal(sites.length, expected, name);
      assert.ok(
        sites.every((site) => site.callee === "primary.get"),
        name,
      );
    }
  },
);

test(
  "compiled CJS and ESM preserve cross-file inline timing guards",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/cross-file-inline-receiver/fixture",
    );
    const cases = [
      ["earlier", 1],
      ["mutated-default", 0],
      ["later", 0],
      ["other-context", 0],
      ["conflicting", 0],
      ["escaped", 0],
    ];
    const reports = cases.map(([name]) => ({
      type: "effects",
      kind: "cache",
      entry: `src/${name}.mts`,
    }));
    const pending = cjs.analyzeProject({ root, reports });
    assert.ok(pending instanceof Promise);
    const result = await pending;
    assert.deepEqual(await esm.analyzeProject({ root, reports }), result);
    for (const [index, [name, expected]] of cases.entries()) {
      const sites = result.reports[index].result.callSites;
      assert.equal(sites.length, expected, name);
      assert.ok(
        sites.every((site) => site.callee === "primary.get"),
        name,
      );
    }
  },
);

// Ordinary discovery skips native behavior; keep its real-addon CI bridge explicit.
test("CI registers inline acquisitions and mutable receiver cases exactly once", () => {
  const workflow = readFileSync(join(__dirname, "../../../.github/workflows/ci.yml"), "utf8");
  const steps = workflow
    .split("      - name: Run real N-API API test")
    .slice(1)
    .map((section) => section.split("\n      - name:")[0]);
  const bridge = readFileSync(join(__dirname, "real-napi-api.test.js"), "utf8");
  assert.ok(bridge.includes('require("./inline-owned-member-api.test.js");'));
  assert.equal(steps.length, 2, "Linux and native platform addon steps");
  for (const step of steps) {
    assert.match(step, /NO_MISTAKES_TEST_NAPI_ADDON_PATH=/u);
    assert.match(step, /pnpm exec vitest run/u);
    assert.ok(step.includes("packages/no-mistakes/scripts/real-napi-api.test.js"));
  }

  const entries = steps.map((step) =>
    [...step.matchAll(/(?:packages|tests)\/[\w./-]+\.test\.(?:js|mjs)/gu)].map(([entry]) => entry),
  );
  const mutable = "packages/no-mistakes/scripts/mutable-receiver-api.test.js";
  const child = spawnSync(
    process.execPath,
    [
      join(__dirname, "../../../fixtures/napi/compiled-test-registration/collect.js"),
      ...new Set([mutable, ...entries.flat()]),
    ],
    { cwd: join(__dirname, "../../.."), encoding: "utf8" },
  );
  assert.equal(child.status, 0, child.stderr);
  const registrations = new Map(JSON.parse(child.stdout).map(({ entry, names }) => [entry, names]));
  const mutableNames = registrations.get(mutable);
  assert.ok(mutableNames.length > 0, "collect actual receiver tests without executing native work");
  assert.equal(new Set(mutableNames).size, mutableNames.length, "receiver test names are unique");
  for (const selected of entries) {
    const names = selected.flatMap((entry) => registrations.get(entry));
    for (const name of mutableNames) {
      assert.equal(
        names.filter((registered) => registered === name).length,
        1,
        `each native CI step must register receiver test exactly once: ${name}`,
      );
    }
  }
});
