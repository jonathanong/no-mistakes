const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

const fixtureRoot = join(
  __dirname,
  "../../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing",
);
const reviewFixtureRoot = join(
  __dirname,
  "../../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-review",
);

function markedLines(source, kinds) {
  return source
    .split("\n")
    .flatMap((line, index) =>
      kinds.some((kind) => line.includes(`// ${kind}:`)) ? [index + 1] : [],
    );
}

function markedFiles(root, files, kinds) {
  return files
    .flatMap((file) =>
      markedLines(readFileSync(join(root, file), "utf8"), kinds).map((line) => [
        "postgres-require-query-annotation",
        file,
        line,
      ]),
    )
    .sort((left, right) => left[1].localeCompare(right[1]) || left[2] - right[2]);
}

test(
  "compiled CJS and ESM trace helper SQL and report unanalyzable executor arguments",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const source = readFileSync(join(fixtureRoot, "src/orders.mts"), "utf8");
    const report = await cjs.check({ root: fixtureRoot });
    assert.deepEqual(await esm.check({ root: fixtureRoot }), report);
    assert.deepEqual(report.warnings, []);
    assert.deepEqual(
      report.rules.map((finding) => [finding.rule, finding.file, finding.line]),
      markedLines(source, ["finding", "unanalyzable"]).map((line) => [
        "postgres-require-query-annotation",
        "src/orders.mts",
        line,
      ]),
    );

    const ignore = await cjs.check({
      root: fixtureRoot,
      config: join(fixtureRoot, ".no-mistakes-ignore.yml"),
    });
    assert.deepEqual(
      await esm.check({
        root: fixtureRoot,
        config: join(fixtureRoot, ".no-mistakes-ignore.yml"),
      }),
      ignore,
    );
    assert.deepEqual(ignore.warnings, []);
    assert.deepEqual(
      ignore.rules.map((finding) => [finding.rule, finding.file, finding.line]),
      markedLines(source, ["finding"]).map((line) => [
        "postgres-require-query-annotation",
        "src/orders.mts",
        line,
      ]),
    );
  },
);

test(
  "compiled CJS and ESM preserve query helper captures and imported export resolution",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const source = readFileSync(join(reviewFixtureRoot, "src/query.mts"), "utf8");
    const reviewedSources = [
      "src/query.mts",
      "src/import-equals.mts",
      "src/local-tag.mts",
      "src/named-tags.mts",
      "src/destructured-tag.mts",
      "src/raw-assertion.ts",
      "src/raw-deleted.mts",
      "src/raw-reassigned.mts",
      "src/raw-shadowed.mts",
    ];
    const report = await cjs.check({ root: reviewFixtureRoot });
    assert.deepEqual(await esm.check({ root: reviewFixtureRoot }), report);
    assert.deepEqual(report.warnings, []);
    assert.deepEqual(
      report.rules.map((finding) => [finding.rule, finding.file, finding.line]),
      markedFiles(reviewFixtureRoot, reviewedSources, ["finding", "unanalyzable"]),
    );
    const transactionLine =
      source.split("\n").findIndex((line) => line.includes("// finding:transaction-same-line")) + 1;
    assert.equal(
      report.rules.filter(
        (finding) => finding.file === "src/query.mts" && finding.line === transactionLine,
      ).length,
      1,
      "BEGIN and an unrelated annotated query on the same line must not hide the configured transaction query",
    );

    const ignore = await cjs.check({
      root: reviewFixtureRoot,
      config: join(reviewFixtureRoot, ".no-mistakes-ignore.yml"),
    });
    assert.deepEqual(
      await esm.check({
        root: reviewFixtureRoot,
        config: join(reviewFixtureRoot, ".no-mistakes-ignore.yml"),
      }),
      ignore,
    );
    assert.deepEqual(ignore.warnings, []);
    assert.deepEqual(
      ignore.rules.map((finding) => [finding.rule, finding.file, finding.line]),
      markedFiles(reviewFixtureRoot, reviewedSources, ["finding"]),
      "a known unannotated imported helper remains a finding when unanalyzableSql is ignored",
    );
  },
);

test(
  "compiled helper tracing uses each importer catalog and honors a forced config",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-monorepo",
    );
    for (const config of [".no-mistakes.yml", ".no-mistakes-report.yml"]) {
      const options = { root, config: join(root, config) };
      const report = await cjs.check(options);
      assert.deepEqual(await esm.check(options), report);
      assert.deepEqual(report.warnings, []);
      assert.deepEqual(
        report.rules.map(({ file, line, target }) => [file, line, target]),
        [["packages/bad/src/query.mts", 3, "annotation"]],
      );
      const batch = await cjs.analyzeProject({ ...options, reports: [{ type: "check" }] });
      assert.deepEqual(batch.reports[0].result, report);
    }
    // Explicit ownership deliberately replaces package-local aliases.
    const forced = { root, tsconfig: join(root, "tsconfig.json") };
    const report = await cjs.check(forced);
    assert.deepEqual(await esm.check(forced), report);
    assert.deepEqual(report.rules, []);
    const batch = await cjs.analyzeProject({ ...forced, reports: [{ type: "check" }] });
    assert.deepEqual(batch.reports[0].result, report);
  },
);

// Keep each saved scenario within the runner's unchanged per-test deadline.
for (const scenario of [
  "helper-tracing-post-merge",
  "helper-tracing-tag-callback",
  "helper-tracing-alternative-callback",
  "helper-tracing-named-delete",
  "helper-tracing-argument-members",
  "helper-tracing-discarded-values",
  "helper-tracing-call-argument-order",
  "helper-tracing-live-binding",
  "helper-tracing-argument-slot-write",
  "helper-tracing-destructuring-alias",
  "helper-tracing-arm-module",
  "helper-tracing-argument-length",
  "helper-tracing-returned-callbacks",
  "helper-tracing-deleted-slot-callback",
  "helper-tracing-module-sibling",
  "helper-tracing-module-sibling-reverse",
  "helper-tracing-callback-installers",
  "helper-tracing-mapped-scalar",
  "helper-tracing-mapped-formal-callback",
]) {
  test(
    `compiled helper value contexts retain parameter and arguments ownership: ${scenario}`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const root = join(
        __dirname,
        `../../../test-cases/rules/postgres-require-query-annotation/fixture/${scenario}`,
      );
      for (const config of [".no-mistakes.yml", ".no-mistakes-ignore.yml"]) {
        const options = { root, config: join(root, config) };
        const report = await cjs.check(options);
        assert.deepEqual(await esm.check(options), report);
        assert.deepEqual(report.warnings, []);
        assert.deepEqual(
          report.rules.map(({ rule, file, line }) => [rule, file, line]),
          markedFiles(
            root,
            scenario === "helper-tracing-arm-module"
              ? ["src/query.mts", "src/state.mts"]
              : scenario === "helper-tracing-mapped-scalar"
                ? ["src/query.cjs"]
                : scenario === "helper-tracing-mapped-formal-callback"
                  ? ["src/query.mts", "src/helper.cjs"]
                  : ["src/query.mts"],
            config.includes("ignore") ? ["finding"] : ["finding", "unanalyzable"],
          ),
        );
        const batch = await cjs.analyzeProject({ ...options, reports: [{ type: "check" }] });
        assert.deepEqual(batch.reports[0].result, report);
      }
    },
  );
}
