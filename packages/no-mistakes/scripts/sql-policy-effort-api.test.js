const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const compiled = addon?.endsWith(".node");
const root = join(__dirname, "../../../fixtures/check/sql-policy-effort");
const rules = ["postgres-explicit-columns", "postgres-required-predicates"];

async function context() {
  const cjs = require("../index.js");
  const esm = await import("../index.mjs");
  const cli =
    process.env.NO_MISTAKES_TEST_CLI_PATH ||
    join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
  return { cjs, esm, cli };
}

function assertSemantics(report, policy) {
  assert.deepEqual(report.warnings, []);
  const definite = report.rules.filter((finding) => finding.target !== "unanalyzable");
  assert.deepEqual(
    definite.map(({ rule, file, line, target }) => ({ rule, file, line, target })),
    rules.flatMap((rule) =>
      [9, 10, 19].map((line) => ({ rule, file: "src/queries.ts", line, target: "accounts" })),
    ),
  );
  // Exact physical witnesses prove direct, const and bounded-helper violations
  // survive ignore, while good direct/const/bind/helper SQL and suppressions stay quiet.
  const uncertain = report.rules.filter((finding) => finding.target === "unanalyzable");
  assert.equal(uncertain.length, policy === "report" ? 4 : 0);
  assert.equal(report.advisories.length, policy === "advise" ? 4 : 0);
  for (const [index, rule] of rules.entries()) {
    const targets = [
      "returning-star, select-star",
      "relation:accounts:predicate:deleted_at IS NULL",
    ];
    for (const [rows, expectedTarget, enabled] of [
      [uncertain, "unanalyzable", policy === "report"],
      [report.advisories, targets[index], policy === "advise"],
    ]) {
      assert.deepEqual(
        rows
          .filter((finding) => finding.rule === rule)
          .map(({ file, line, target }) => ({ file, line, target })),
        enabled
          ? [25, 26].map((line) => ({ file: "src/queries.ts", line, target: expectedTarget }))
          : [],
      );
    }
  }
}

function assertLowRewrite(report, selectedRules = rules) {
  const residual = [...report.rules, ...report.advisories].filter(
    (finding) => finding.target !== "accounts" && selectedRules.includes(finding.rule),
  );
  assert.ok(residual.length > 0);
  for (const finding of residual) {
    assert.match(finding.message, /effort low/);
    assert.match(finding.message, /literal string/);
    assert.match(finding.message, /one local const/);
  }
}

function cliCheck(cli, config, effort) {
  const args = ["check", "--root", root, "--config", config, "--format", "json"];
  if (effort) args.push("--effort", effort);
  const child = spawnSync(cli, args, { encoding: "utf8" });
  assert.equal(child.error, undefined);
  assert.equal(child.status, 1, child.stderr);
  return JSON.parse(child.stdout);
}

// Separate policy tests keep each native test below the runner's duration gate.
for (const policy of ["report", "advise", "ignore"]) {
  test(
    `compiled SQL policy effort preserves async CJS, ESM, batch and CLI parity: ${policy}`,
    { skip: !compiled },
    async () => {
      const { cjs, esm, cli } = await context();
      const options = { root, config: join(root, `low-${policy}.yml`) };
      const pending = cjs.check(options);
      assert.ok(pending instanceof Promise);
      const low = await pending;
      assertSemantics(low, policy);
      if (policy !== "ignore") assertLowRewrite(low);
      assert.deepEqual(await esm.check(options), low);
      assert.deepEqual(cliCheck(cli, options.config), low);
      const max = await cjs.check({ ...options, effort: "max" });
      assertSemantics(max, policy);
      const request = {
        ...options,
        reports: [
          { type: "check", id: "max-first", effort: "max" },
          { type: "check", id: "low", effort: "low" },
          { type: "check", id: "inherited-low" },
        ],
      };
      const batchPending = cjs.analyzeProject(request);
      assert.ok(batchPending instanceof Promise);
      const batch = await batchPending;
      assert.equal(batch.reports.length, 3);
      for (const report of batch.reports) {
        assert.deepEqual(report.result, report.id === "max-first" ? max : low);
      }
      assert.deepEqual(await esm.analyzeProject(request), batch);
    },
  );
}

test(
  "compiled SQL policy default and explicit max remain identical",
  { skip: !compiled },
  async () => {
    const { cjs, esm, cli } = await context();
    const options = { root, config: join(root, "default.yml") };
    const expected = await cjs.check(options);
    assertSemantics(expected, "report");
    assert.deepEqual(await cjs.check({ ...options, effort: "max" }), expected);
    assert.deepEqual(await esm.check(options), expected);
    assert.deepEqual(cliCheck(cli, options.config), expected);
    assert.deepEqual(cliCheck(cli, join(root, "max.yml")), expected);
  },
);

test(
  "compiled SQL policy applications keep independent low and max effort",
  { skip: !compiled },
  async () => {
    const { cjs, esm, cli } = await context();
    const config = join(root, "mixed.yml");
    const options = { root, config };
    const mixed = await cjs.check(options);
    assertSemantics(mixed, "advise");
    assertLowRewrite(mixed, [rules[0]]);
    const low = await cjs.check({ root, config: join(root, "low-advise.yml") });
    const max = await cjs.check({ root, config: join(root, "low-advise.yml"), effort: "max" });
    for (const field of ["rules", "advisories"]) {
      for (const [rule, expected] of [
        [rules[0], low],
        [rules[1], max],
      ]) {
        assert.deepEqual(
          mixed[field].filter((finding) => finding.rule === rule),
          expected[field].filter((finding) => finding.rule === rule),
        );
      }
    }
    assert.deepEqual(await esm.check(options), mixed);
    assert.deepEqual(cliCheck(cli, config), mixed);
    const request = { ...options, reports: [{ type: "check", id: "mixed" }] };
    const batch = await cjs.analyzeProject(request);
    assert.deepEqual(batch.reports[0].result, mixed);
    assert.deepEqual(await esm.analyzeProject(request), batch);
  },
);

test(
  "compiled SQL policy messages and suppression retain existing behavior",
  { skip: !compiled },
  async () => {
    const { cjs, esm, cli } = await context();
    const options = { root, config: join(root, "custom.yml") };
    const low = await cjs.check(options);
    assertSemantics(low, "ignore");
    // These legacy rules retain their built-in messages even with RuleDef.message.
    // Effort adoption must not introduce a low-only custom-message behavior change.
    assert.deepEqual(low.rules, (await cjs.check({ ...options, effort: "max" })).rules);
    assert.match(low.rules[0].message, /SELECT \*/);
    assert.match(low.rules[3].message, /deleted_at IS NULL/);
    assert.deepEqual(await esm.check(options), low);
    assert.deepEqual(cliCheck(cli, options.config), low);
  },
);
