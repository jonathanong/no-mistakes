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
