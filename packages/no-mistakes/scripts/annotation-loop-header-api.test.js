const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;

test(
  "compiled low annotation separates one-time loop headers from repeated queries",
  { skip: !addon?.endsWith(".node") },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-require-query-annotation/fixture/low-loop-headers",
    );
    const source = readFileSync(join(root, "src/query.mts"), "utf8");
    const marked = (marker) =>
      source.split("\n").flatMap((line, index) => (line.includes(marker) ? [index + 1] : []));
    const missing = marked("EXPECT_MISSING_");
    for (const policy of ["report", "advise", "ignore"]) {
      const config = join(root, `.no-mistakes-low-${policy}.yml`);
      const report = await cjs.check({ root, config });
      assert.deepEqual(
        report.rules.map((row) => row.line),
        missing,
      );
      assert.deepEqual(report.advisories, []);
      assert.deepEqual(report.warnings, []);
      assert.deepEqual(await esm.check({ root, config }), report);
    }
    const maximum = await cjs.check({ root, config: join(root, ".no-mistakes-max-report.yml") });
    for (const line of marked("EXPECT_ACCEPTED_")) {
      assert.ok(!maximum.rules.some((row) => row.line === line), `max header ${line}`);
    }
    for (const line of marked("EXPECT_MAX_UNKNOWN")) {
      assert.ok(maximum.rules.some((row) => row.line === line && /unanalyzable/.test(row.message)));
    }
  },
);
