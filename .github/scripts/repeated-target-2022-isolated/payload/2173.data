const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled async CJS and ESM discover opaque array callback effects conservatively",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const source = readFileSync(
      join(
        __dirname,
        "../../../fixtures/postgres-facts/executed-source/control-flow-array-callbacks.ts",
      ),
      "utf8",
    );
    const options = {
      sources: [{ fileName: "control-flow-array-callbacks.ts", source }],
      importSpecifier: "@example/db",
      resolveImports: true,
      trustedSqlTags: [{ module: "@example/db", name: "sql" }],
    };
    const pending = cjs.extractExecutedPostgresSql(options);
    assert.equal(typeof pending.then, "function");
    const rows = await pending;
    assert.deepEqual(await esm.extractExecutedPostgresSql(options), rows);
    assert.equal(rows.length, 1);
    assert.equal(rows[0].complete, false);
    assert.equal(rows[0].onlyValuePlaceholders, false);
    assert.equal(rows[0].sqlText, null);
    assert.equal(rows[0].incompleteReason, "unresolved-value-or-effect");
  },
);
