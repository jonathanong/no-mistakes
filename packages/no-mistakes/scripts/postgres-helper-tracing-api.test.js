const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

const fixtureRoot = join(
  __dirname,
  "../../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing",
);

function markedLines(source, kinds) {
  return source
    .split("\n")
    .flatMap((line, index) =>
      kinds.some((kind) => line.includes(`// ${kind}:`)) ? [index + 1] : [],
    );
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
