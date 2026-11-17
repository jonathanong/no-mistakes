import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";
import { expect, it } from "vitest";
import { __dirname, fixture, plugin } from "./helpers.mjs";

const compatibilityOptions = {
  importSpecifier: "@example/psql",
  executorFactoryNames: ["beginTransaction", "beginBoundedTransaction"],
  executorTypeNames: ["TransactionQuery", "QueryExecutor", "PoolClient", "QueryOptions"],
  executorNames: ["readPool", "writePool"],
  trustedSqlTags: [{ module: "sql-template-strings", name: "default" }],
};
const existingOptions = {
  importSpecifier: "@example/db",
  executorFactoryNames: ["withQuery"],
  executorTypeNames: ["QueryExecutor"],
  executorNames: ["read", "write"],
  trustedSqlTags: [{ module: "@example/sql", name: "sql" }],
};
const cases = [
  { file: "default-compat.ts", typed: false },
  { file: "default-compat.ts", typed: true },
  { file: "local.ts", typed: false },
  { file: "local.ts", typed: true },
  { file: "imported.ts", typed: false },
  { file: "imported.ts", typed: true },
  { file: "options.ts", typed: false },
  { file: "options.ts", typed: true },
  { file: "evaluated.ts", typed: false },
  { file: "unsafe.ts", typed: false },
  { file: "writes.ts", typed: false },
];

function findings(name, typed) {
  const filename = resolve(
    __dirname,
    "../../../test-cases/eslint-snippets/fixture/executor-target",
    name,
  );
  const linter = new Linter({ configType: "flat", cwd: dirname(filename) });
  return linter
    .verify(
      fixture(`executor-target/${name}`),
      {
        files: ["**"],
        languageOptions: {
          parser: tsParser,
          ecmaVersion: 2024,
          sourceType: "module",
          parserOptions: typed ? { projectService: true, tsconfigRootDir: dirname(filename) } : {},
        },
        plugins: { "no-mistakes": plugin },
        rules: {
          "no-mistakes/postgres-no-executor-adapter": [
            "error",
            name === "default-compat.ts" ? compatibilityOptions : existingOptions,
          ],
        },
      },
      filename,
    )
    .map(({ line, column, endLine, endColumn, ruleId, messageId, message }) => ({
      line,
      column,
      endLine,
      endColumn,
      ruleId,
      messageId,
      message,
    }));
}

it("keeps default-mode diagnostics byte-identical to plugin v0.109.68", async () => {
  const report = `${JSON.stringify(
    cases.map(({ file, typed }) => ({ file, typed, findings: findings(file, typed) })),
    null,
    2,
  )}\n`;
  await expect(report).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-default-v0.109.68.snap",
  );
});
