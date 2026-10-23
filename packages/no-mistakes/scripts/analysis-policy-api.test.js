const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const compiled = addon?.endsWith(".node");
require("./annotation-assignment-api.test.js");
require("./annotation-loop-header-api.test.js");
require("./annotation-reference-api.test.js");

async function policyContext() {
  const cjs = require("../index.js");
  const esm = await import("../index.mjs");
  const root = join(
    __dirname,
    "../../../test-cases/codebase-analysis/rule-analysis-policy/fixture",
  );
  const cli =
    process.env.NO_MISTAKES_TEST_CLI_PATH ||
    join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
  return { cjs, esm, root, cli };
}

for (const [policy, failures, advisories] of [
  ["report", 1, 0],
  ["advise", 0, 1],
  ["ignore", 0, 0],
])
  test(
    `compiled check effort and shared policies preserve CLI and async API parity: ${policy}`,
    { skip: !compiled },
    async () => {
      const { cjs, esm, root, cli } = await policyContext();
      const options = { root, config: join(root, `sql-${policy}.yml`) };
      const pending = cjs.check(options);
      assert.ok(pending instanceof Promise);
      const expected = await pending;
      assert.equal(expected.rules.length, failures);
      assert.equal(expected.advisories.length, advisories);
      const low = await cjs.check({ ...options, effort: "low" });
      assert.equal(low.rules.length, failures);
      assert.equal(low.advisories.length, advisories);
      const locations = (rows) => rows.map(({ rule, file, line }) => ({ rule, file, line }));
      assert.deepEqual(locations(low.rules), locations(expected.rules));
      assert.deepEqual(locations(low.advisories), locations(expected.advisories));
      for (const finding of [...low.rules, ...low.advisories]) {
        assert.match(finding.message, /query\("\/\* name \*\/ " \+ sql\)/);
        assert.match(finding.message, /one local const/);
      }
      const expectedByEffort = { max: expected, low };
      let defaultStdout;
      for (const effort of [undefined, "max", "low"]) {
        const requested = { ...options, ...(effort ? { effort } : {}) };
        const effortExpected = expectedByEffort[effort || "max"];
        for (const api of [cjs, esm]) {
          // Default and low CJS calls above already exercised these exact options.
          if (api !== cjs || effort === "max")
            assert.deepEqual(await api.check(requested), effortExpected);
          assert.deepEqual(
            await api.check({ ...requested, config: join(root, `legacy-sql-${policy}.yml`) }),
            effortExpected,
          );
        }
        const args = ["check", "--root", root, "--config", options.config, "--format", "json"];
        if (effort) args.push("--effort", effort);
        const child = spawnSync(cli, args, { encoding: "utf8", env: process.env });
        assert.equal(child.error, undefined);
        assert.equal(child.status, failures ? 1 : 0, child.stderr);
        assert.deepEqual(JSON.parse(child.stdout), effortExpected);
        if (effort === undefined) defaultStdout = child.stdout;
        else if (effort === "max") assert.equal(child.stdout, defaultStdout);
      }
      // Each report has a distinct override: sharing a prepared result must not
      // make the first report's effort become the next report's option.
      const request = {
        ...options,
        reports: ["low", "max", undefined, "low"].map((effort, index) => ({
          type: "check",
          id: String(index),
          ...(effort ? { effort } : {}),
        })),
      };
      const batchPending = cjs.analyzeProject(request);
      assert.ok(batchPending instanceof Promise);
      const batch = await batchPending;
      assert.equal(batch.reports.length, request.reports.length);
      for (const report of batch.reports) {
        const effort = request.reports[Number(report.id)].effort || "max";
        assert.deepEqual(report.result, expectedByEffort[effort]);
      }
      assert.deepEqual(await esm.analyzeProject(request), batch);
    },
  );

test("compiled legacy policies preserve project inheritance", { skip: !compiled }, async () => {
  const { cjs, esm, root } = await policyContext();
  const inherited = await cjs.check({ root, config: "project-no-policy.yml" });
  assert.deepEqual(
    inherited.rules.map(({ rule, line }) => [rule, line]),
    [
      ["postgres-no-mutable-sql-registry", 4],
      ["postgres-no-unbounded-sql-identifier", 5],
    ],
  );
  for (const config of ["project-fail.yml", "project-previously-ignored.yml", "opaque-legacy.yml"])
    for (const api of [cjs, esm]) assert.deepEqual(await api.check({ root, config }), inherited);
});

test("compiled checks reject unsupported effort", { skip: !compiled }, async () => {
  const { cjs, root } = await policyContext();
  await assert.rejects(cjs.check({ root, effort: "high" }), /effort|variant|high/i);
});

for (const config of [
  "shared-invalid-annotation.yml",
  "shared-invalid-sql.yml",
  "shared-invalid-common.yml",
])
  test(
    `compiled shared policies reject invalid options: ${config}`,
    { skip: !compiled },
    async () => {
      const { cjs, esm, root, cli } = await policyContext();
      for (const api of [cjs, esm])
        await assert.rejects(api.check({ root, config }), /unanalyzableSql/);
      const child = spawnSync(cli, ["check", "--root", root, "--config", join(root, config)], {
        encoding: "utf8",
        env: process.env,
      });
      assert.equal(child.error, undefined);
      assert.equal(child.status, 2, child.stderr);
      assert.match(child.stderr, /unanalyzableSql/);
    },
  );

test(
  "compiled shared policy precedence and non-check effort remain stable",
  { skip: !compiled },
  async () => {
    const { cjs, esm, root } = await policyContext();
    const precedence = await cjs.check({ root, config: "shared-valid-annotation.yml" });
    assert.deepEqual(precedence, await cjs.check({ root, config: "sql-advise.yml" }));
    // ProjectOptions already accepts effort outside check; those commands ignore it.
    for (const api of [cjs, esm])
      assert.deepEqual(
        await api.resolveConfig({ root, effort: "low" }),
        await api.resolveConfig({ root }),
      );
  },
);

test(
  "compiled low annotation accepts default tags and preserves warnings and suppressions",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const cli =
      process.env.NO_MISTAKES_TEST_CLI_PATH ||
      join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-require-query-annotation/fixture/low-heads",
    );
    const warningRoot = join(
      __dirname,
      "../../../test-cases/codebase-analysis/rule-analysis-policy/fixture",
    );
    for (const [options, locations, warningCount] of [
      [
        { root },
        [
          ["default-tags.mts", 7],
          ["default-tags.mts", 9],
        ],
        0,
      ],
      [{ root: warningRoot, config: "sql-tags.yml" }, [], 1],
      [{ root: warningRoot, config: "sql-tags.yml", effort: "max" }, [], 0],
    ]) {
      const pending = cjs.check(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(
        report.rules.map(({ file, line }) => [file, line]),
        locations,
      );
      assert.deepEqual(report.advisories, []);
      assert.equal(report.warnings.length, warningCount);
      if (warningCount)
        assert.match(report.warnings[0], /trustedSqlTags is deprecated and ignored at effort low/);
      assert.deepEqual(await esm.check(options), report);
      const args = ["check", "--root", options.root, "--format", "json"];
      if (options.config) args.push("--config", join(options.root, options.config));
      if (options.effort) args.push("--effort", options.effort);
      const child = spawnSync(cli, args, { encoding: "utf8", env: process.env });
      assert.equal(child.error, undefined);
      assert.equal(child.status, locations.length ? 1 : 0, child.stderr);
      assert.deepEqual(JSON.parse(child.stdout), report);
    }
  },
);
