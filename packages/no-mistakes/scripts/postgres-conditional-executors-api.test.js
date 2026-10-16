const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled conditional executor aliases retain lexical ownership in CJS and ESM",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const source = readFileSync(
      join(__dirname, "../../../fixtures/postgres-facts/executed-source/conditional-executors.ts"),
      "utf8",
    );
    for (const resolveImports of [false, true]) {
      const options = {
        sources: [{ fileName: "conditional-executors.ts", source }],
        importSpecifier: "@example/db",
        executorNames: ["read", "write"],
        resolveImports,
      };
      const facts = await cjs.extractExecutedPostgresSql(options);
      assert.deepEqual(await esm.extractExecutedPostgresSql(options), facts);
      assert.deepEqual(
        facts.map((fact) => fact.annotation),
        ["assigned", "direct", "nested", "outer", "deep", "outside-class"],
      );
      for (const fact of facts) {
        assert.equal(fact.fileName, "conditional-executors.ts");
        assert.equal(fact.complete, true);
        assert.ok(fact.sqlText.includes("SELECT id FROM items"));
        assert.ok(source.split("\n")[fact.line - 1].includes(fact.annotation));
      }
    }
  },
);
