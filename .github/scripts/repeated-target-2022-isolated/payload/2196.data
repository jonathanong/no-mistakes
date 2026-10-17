const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled CJS and ESM retain function-scoped SQL var annotation checks",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-require-query-annotation/fixture/var-scopes",
    );
    const source = readFileSync(join(root, "src/query.ts"), "utf8");
    const expected = source
      .split("\n")
      .flatMap((line, index) => (line.includes("// finding:") ? [index + 1] : []));
    const report = await cjs.check({ root });
    assert.deepEqual(await esm.check({ root }), report);
    assert.deepEqual(report.warnings, []);
    assert.deepEqual(
      report.rules.map((finding) => [finding.rule, finding.file, finding.line]),
      expected.map((line) => ["postgres-require-query-annotation", "src/query.ts", line]),
    );
  },
);
