const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
test(
  "compiled mutable heads preserve scalar reads and reject tagged assignment escapes",
  { skip: !addon?.endsWith(".node") },
  async () => {
    const api = require("../index.js");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-require-query-annotation/fixture/low-reference-uses",
    );
    const source = readFileSync(join(root, "src/query.ts"), "utf8");
    const marked = (marker) =>
      source.split("\n").flatMap((line, i) => (line.includes(marker) ? [i + 1] : []));
    const missing = marked("EXPECT_MISSING_");
    const unknown = marked("EXPECT_UNKNOWN_");
    for (const policy of ["report", "advise", "ignore"]) {
      const result = await api.check({
        root,
        config: join(root, `.no-mistakes-low-${policy}.yml`),
      });
      const expected =
        policy === "report" ? [...missing, ...unknown].sort((a, b) => a - b) : missing;
      assert.deepEqual(
        result.rules.map((row) => row.line),
        expected,
      );
      assert.deepEqual(
        result.advisories.map((row) => row.line),
        policy === "advise" ? unknown : [],
      );
      assert.deepEqual(result.warnings, []);
    }
  },
);
