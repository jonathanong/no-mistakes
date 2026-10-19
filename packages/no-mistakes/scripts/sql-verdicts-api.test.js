const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled SQL verdict checks preserve CJS, ESM, suppression, and batched isolation",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-require-query-annotation/fixture/provenance-advice",
    );
    const options = { root, config: join(root, ".no-mistakes.yml") };
    const baseline = await cjs.check(options);
    assert.equal(Object.hasOwn(baseline, "sqlVerdicts"), false);
    assert.equal(baseline.rules.length, 0);
    assert.equal(baseline.advisories.length, 1);

    const requested = { ...options, includeSqlVerdicts: true };
    const pending = cjs.check(requested);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(await esm.check(requested), report);
    assert.deepEqual(report.rules, baseline.rules);
    assert.deepEqual(report.advisories, baseline.advisories);
    assert.equal(report.sqlVerdicts.length, 5);
    assert.equal(report.sqlVerdicts.filter((row) => row.file !== "src/query.ts").length, 2);

    const batch = await cjs.analyzeProject({
      ...options,
      reports: [
        { type: "check", id: "first", includeSqlVerdicts: true },
        { type: "check", id: "plain" },
        { type: "check", id: "again", includeSqlVerdicts: true },
      ],
    });
    assert.deepEqual(batch.reports[0].result, report);
    assert.deepEqual(batch.reports[1].result, baseline);
    assert.deepEqual(batch.reports[2].result, report);
  },
);

test(
  "compiled SQL verdicts identify overlapping named applications",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-sql-verdicts/fixture/overlapping-applications",
    );
    const report = await cjs.check({
      root,
      config: join(root, ".no-mistakes.yml"),
      includeSqlVerdicts: true,
    });
    const rows = report.sqlVerdicts.filter((row) => row.subcheck === "select-star");
    assert.deepEqual(
      rows.map(({ application, status }) => [application.index, application.name, status]),
      [
        [0, "strict", "violation"],
        [1, "permissive", "proven-clear"],
      ],
    );
  },
);

test(
  "compiled CJS and ESM report a conflict inside an executed CTE",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-sql-verdicts/fixture/consumer-cte-conflict",
    );
    for (const config of ["fail.yml", ".no-mistakes.yml", "advise.yml"]) {
      const options = { root, config: join(root, config) };
      const plain = await cjs.check(options);
      const detailed = await cjs.check({ ...options, includeSqlVerdicts: true });
      assert.deepEqual(detailed.rules, plain.rules, config);
      assert.deepEqual(detailed.advisories, plain.advisories, config);
      assert.deepEqual(await esm.check({ ...options, includeSqlVerdicts: true }), detailed);
      assert.deepEqual(
        detailed.rules.map(({ file, line, target }) => [file, line, target]),
        [["src/query.ts", 4, "missing-canonical-order"]],
      );
      assert.equal(detailed.sqlVerdicts.length, 1);
      assert.equal(detailed.sqlVerdicts[0].status, "violation");
      assert.equal(detailed.sqlVerdicts[0].evidence, "exact");
    }
  },
);
