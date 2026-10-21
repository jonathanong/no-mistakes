import assert from "node:assert/strict";
import { Linter } from "eslint";
import { describe, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { fixture, require } from "./helpers.mjs";

const {
  isModuleConstSql,
  moduleConstSqlDeclaration,
} = require("../src/rules/postgres-no-async-sql-builder-constants.js");

function probeModuleConst(code) {
  const linter = new Linter({ configType: "flat" });
  const seen = [];
  const probe = {
    create(context) {
      return {
        "CallExpression[callee.name='run'] > Identifier"(node) {
          if (node.parent.callee === node) return;
          seen.push({
            name: node.name,
            isSql: isModuleConstSql(node, context),
            declaration: moduleConstSqlDeclaration(node, context) !== null,
          });
        },
      };
    },
  };
  linter.verify(
    code,
    {
      files: ["**/*.ts"],
      languageOptions: {
        ecmaVersion: 2024,
        sourceType: "module",
        parser: tsParser,
      },
      plugins: { probe: { rules: { probe } } },
      rules: { "probe/probe": "error" },
    },
    { filename: "app.ts" },
  );
  return seen;
}

describe("isModuleConstSql", () => {
  it("accepts a module const string and an alias of one", () => {
    assert.deepEqual(
      probeModuleConst(fixture("postgres-no-async-sql-builder/module-const-sql-string.ts")),
      [{ name: "Q", isSql: true, declaration: true }],
    );
    assert.deepEqual(
      probeModuleConst(fixture("postgres-no-async-sql-builder/module-const-sql-alias.ts")),
      [{ name: "Q", isSql: true, declaration: true }],
    );
  });

  it("rejects a module let string", () => {
    assert.deepEqual(
      probeModuleConst(fixture("postgres-no-async-sql-builder/module-let-sql-string.ts")),
      [{ name: "Q", isSql: false, declaration: false }],
    );
  });
});
