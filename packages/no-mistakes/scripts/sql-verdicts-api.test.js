const assert = require("node:assert/strict");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

for (const name of [
  "class-method",
  "this-method",
  "object-method",
  "property-function",
  "named-property-function",
  "unused-method",
  "method-forwarding",
])
  test(
    `compiled returned SQL retains conservative member recall: ${name}`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const root = join(
        __dirname,
        "../../../test-cases/rules/postgres-sql-shape-policy/fixture/returned-member-owners",
      );
      const options = { root, config: join(root, `${name}.yml`) };
      const pending = cjs.check(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(await esm.check(options), report);
      assert.deepEqual(
        report.rules.map(({ file, line, target }) => [file, line, target]),
        [[`src/${name}.ts`, 4, "literal-limit"]],
      );
      const prepared = await cjs.analyzeProject({
        ...options,
        reports: [{ type: "check", id: "check" }],
      });
      assert.deepEqual(prepared.reports[0].result, report);
    },
  );

for (const [state, expected] of [
  ["live", 2],
  ["dropped", 0],
  ["replaced", 0],
])
  test(
    `compiled application inserts use the final trigger state: ${state}`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const root = join(
        __dirname,
        "../../../test-cases/rules/postgres-idempotent-insert/fixture",
        `embedded-trigger-final-${state}`,
      );
      const options = { root, config: join(root, ".no-mistakes.yml"), includeSqlVerdicts: true };
      const pending = cjs.check(options);
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(await esm.check(options), report);
      assert.equal(report.rules.length, expected);
      if (expected) {
        assert.deepEqual(
          report.rules.map(({ file, line }) => [file, line]),
          [
            ["app/q.ts", 3],
            ["lib/q.ts", 3],
          ],
        );
        for (const finding of report.rules)
          assert.match(finding.message, /fires before conflict checks/);
      }
      const triggers = report.sqlVerdicts.filter((row) => row.subcheck === "trigger-replay");
      assert.equal(triggers.length, 2);
      for (const row of triggers) assert.equal(row.status, expected ? "violation" : "proven-clear");
      const batch = await cjs.analyzeProject({
        root,
        config: options.config,
        reports: [{ type: "check", id: "check", includeSqlVerdicts: true }],
      });
      assert.deepEqual(batch.reports[0].result, report);
    },
  );

test(
  "compiled SQL returns retain callable bindings, await, and lexical shadows",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-sql-shape-policy/fixture/returned-call-identities",
    );
    const options = { root, config: join(root, ".no-mistakes.yml") };
    const pending = cjs.check(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(await esm.check(options), report);
    assert.deepEqual(
      report.rules.map(({ file, line, target }) => [file, line, target]),
      [
        ["src/await.ts", 3, "literal-limit"],
        ["src/named.ts", 3, "literal-limit"],
        ["src/recursive.ts", 6, "literal-limit"],
        ["src/shadow-edge.ts", 5, "literal-limit"],
        ["src/shadow.ts", 5, "literal-limit"],
      ],
    );
    const batch = await cjs.analyzeProject({
      ...options,
      reports: [{ type: "check", id: "check" }],
    });
    assert.deepEqual(batch.reports[0].result, report);
    const strict = await cjs.check({ root, config: join(root, "strict.yml") });
    assert.deepEqual(
      strict.rules.map(({ file, line }) => [file, line]),
      [
        ["src/shadow.ts", 5],
        ["src/shadow.ts", 6],
      ],
    );
  },
);

const identifierCases = [
  ["fail-array-destructured-row", [[5, "id"]]],
  ["fail-array-destructured-row-elision", [[4, "schema"]]],
  ["fail-comma-env", [[3, "process.env.SCHEMA"]]],
  ["pass-array-row-overwritten-member", []],
  ["fail-array-row-untouched-member", [[5, "first"]]],
  ["fail-array-row-earlier-read", [[4, "first"]]],
  ["fail-array-row-copied-member", [[6, "schema"]]],
  ["fail-array-row-env-write", [[5, "first"]]],
  ["fail-array-row-conditional-write", [[6, "first"]]],
  ["fail-array-row-alias-write-barrier", [[7, "first"]]],
  ["fail-array-row-call-barrier", [[7, "first"]]],
  ["fail-array-row-object-write-barrier", [[6, "first"]]],
  ["fail-array-row-interpolation-effect-barrier", [[6, "first"]]],
  ["fail-array-row-shadow-scope", [[9, "first"]]],
  [
    "fail-array-row-unmatched-overwrites",
    [
      [6, "first"],
      [8, "first"],
      [10, "first"],
      [14, "first"],
    ],
  ],
  ["pass-comma-discarded-env", []],
  ["pass-array-destructured-http-row", []],
  ["pass-array-destructured-mutable-row", []],
  ["pass-array-destructured-mutable-result", []],
  ["pass-empty-and-computed-patterns", []],
];

for (const [scenario, expected] of identifierCases)
  test(
    `compiled identifier checks retain array rows and comma result provenance: ${scenario}`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
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

for (const config of ["fail.yml", ".no-mistakes.yml", "advise.yml"])
  test(
    `compiled CJS and ESM report a conflict inside an executed CTE: ${config}`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const root = join(
        __dirname,
        "../../../test-cases/rules/postgres-sql-verdicts/fixture/consumer-cte-conflict",
      );
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
    },
  );

for (const configName of [".no-mistakes.yml", "fail.yml"])
  for (const includeSqlVerdicts of [false, true])
    test(
      `compiled advice matches CLI: ${configName}, ledger ${includeSqlVerdicts}`,
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
          const config = join(root, configName);
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
