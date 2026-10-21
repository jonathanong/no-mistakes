const assert = require("node:assert/strict");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled identifier checks retain array rows and comma result provenance",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    for (const [scenario, expected] of [
      ["fail-array-destructured-row", [[5, "id"]]],
      ["fail-array-destructured-row-elision", [[4, "schema"]]],
      ["fail-comma-env", [[3, "process.env.SCHEMA"]]],
      ["pass-comma-discarded-env", []],
      ["pass-array-destructured-http-row", []],
      ["pass-array-destructured-mutable-row", []],
      ["pass-array-destructured-mutable-result", []],
      ["pass-empty-and-computed-patterns", []],
    ]) {
      const root = join(
        __dirname,
        "../../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture",
        scenario,
      );
      const options = { root, config: join(root, ".no-mistakes.yml") };
      const pending = cjs.check(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(await esm.check(options), report, scenario);
      const found = report.rules.filter(
        (finding) => finding.rule === "postgres-no-unbounded-sql-identifier",
      );
      assert.deepEqual(
        found.map((finding) => [finding.line, finding.target]),
        expected,
        scenario,
      );
      for (const finding of found) {
        assert.equal(finding.file, "src/query.ts", scenario);
        assert.match(finding.message, /switch whose default throws/, scenario);
      }
      const batch = await cjs.analyzeProject({
        ...options,
        reports: [{ type: "check", id: "check" }],
      });
      assert.deepEqual(batch.reports[0].result, report, scenario);
    }
  },
);

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

test(
  "compiled advice aggregates obligations and matches CLI with either ledger flag",
  { skip: !compiled },
  async () => {
    const { execFileSync } = require("node:child_process");
    const { mkdtempSync, rmSync } = require("node:fs");
    const { tmpdir } = require("node:os");
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-sql-verdicts/fixture/advisory-aggregation",
    );
    const cli =
      process.env.NO_MISTAKES_TEST_CLI_PATH ||
      join(
        dirname(process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH),
        process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes",
      );
    const runtime = mkdtempSync(join(tmpdir(), "no-mistakes-sql-advice-"));
    try {
      for (const configName of [".no-mistakes.yml", "fail.yml"]) {
        const config = join(root, configName);
        for (const includeSqlVerdicts of [false, true]) {
          const pending = cjs.check({ root, config, includeSqlVerdicts });
          assert.ok(pending instanceof Promise);
          const report = await pending;
          assert.deepEqual(await esm.check({ root, config, includeSqlVerdicts }), report);
          const args = ["check", "--root", root, "--config", config, "--format", "json"];
          if (includeSqlVerdicts) args.push("--include-sql-verdicts");
          // This saved fixture deliberately contains proven violations (exit 1).
          let stdout;
          try {
            stdout = execFileSync(cli, args, {
              env: { ...process.env, XDG_RUNTIME_DIR: runtime },
              encoding: "utf8",
            });
          } catch (error) {
            assert.equal(error.status, 1);
            stdout = error.stdout;
          }
          assert.deepEqual(report, JSON.parse(stdout));
          if (configName === "fail.yml") {
            assert.equal(report.advisories.length, 0);
            assert.ok(
              report.rules.some(({ line, target }) => line === 10 && target === "unanalyzable-sql"),
            );
          } else {
            assert.equal(report.advisories.length, 12);
            assert.equal(report.advisories.filter(({ line }) => line === 7).length, 2);
            assert.equal(report.advisories.filter(({ line }) => line === 13).length, 4);
          }
          assert.ok(
            report.rules.some(({ line, target }) => line === 9 && target === "CREATE TABLE"),
          );
          if (includeSqlVerdicts) assert.equal(report.sqlVerdicts.length, 84);
        }
      }
    } finally {
      rmSync(runtime, { recursive: true, force: true });
    }
  },
);

test(
  "borrowed coincident applications preserve full async reports and independent sinks",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-sql-verdicts/fixture/borrowed-applications",
    );
    const options = { root, config: join(root, ".no-mistakes.yml"), includeSuppressed: true };
    const baseline = await cjs.check(options);
    const requested = { ...options, includeSqlVerdicts: true };
    const pending = cjs.check(requested);
    assert.ok(pending instanceof Promise);
    const [first, second, moduleReport] = await Promise.all([
      pending,
      cjs.check(requested),
      esm.check(requested),
    ]);
    assert.deepEqual(second, first);
    assert.deepEqual(moduleReport, first);
    const { spawnSync } = require("node:child_process");
    const { mkdtempSync, rmSync } = require("node:fs");
    const { tmpdir } = require("node:os");
    const cli =
      process.env.NO_MISTAKES_TEST_CLI_PATH ||
      join(
        dirname(process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH),
        process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes",
      );
    const runtime = mkdtempSync(join(tmpdir(), "no-mistakes-borrowed-sql-"));
    try {
      for (const includeSqlVerdicts of [false, true]) {
        const args = [
          "check",
          "--root",
          root,
          "--config",
          options.config,
          "--format",
          "json",
          "--include-suppressed",
        ];
        if (includeSqlVerdicts) args.push("--include-sql-verdicts");
        const child = spawnSync(cli, args, {
          env: { ...process.env, XDG_RUNTIME_DIR: runtime },
          encoding: "utf8",
        });
        assert.equal(child.status, 1, child.stderr);
        assert.deepEqual(JSON.parse(child.stdout), includeSqlVerdicts ? first : baseline);
      }
    } finally {
      rmSync(runtime, { recursive: true, force: true });
    }
    const { sqlVerdicts, ...otherFields } = first;
    assert.deepEqual(otherFields, baseline);
    assert.equal(baseline.rules.length, 1);
    assert.equal(baseline.suppressed.length, 1);
    assert.ok(baseline.advisories.length > 0);
    const rows = sqlVerdicts.filter((row) => row.subcheck === "select-star");
    for (const index of [0, 1, 2]) {
      const applicationRows = rows.filter((row) => row.application.index === index);
      assert.ok(applicationRows.some((row) => row.status === "abstain" && row.advisory));
      assert.ok(
        applicationRows.some((row) => row.status === (index === 2 ? "proven-clear" : "violation")),
      );
      assert.deepEqual(
        applicationRows[0].application,
        index === 2 ? { index } : { index, name: "coincident" },
      );
    }
    const batch = await cjs.analyzeProject({
      root,
      config: options.config,
      reports: [
        { type: "check", includeSqlVerdicts: true, includeSuppressed: true },
        { type: "check", includeSuppressed: true },
        { type: "check", includeSqlVerdicts: true, includeSuppressed: true },
      ],
    });
    assert.deepEqual(
      batch.reports.map(({ result }) => result),
      [first, baseline, first],
    );
  },
);
