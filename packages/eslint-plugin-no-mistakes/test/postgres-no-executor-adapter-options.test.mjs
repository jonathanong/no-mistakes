import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const filename = resolve(
  __dirname,
  "../../../test-cases/eslint-snippets/fixture/executor-target/options.ts",
);
const code = fixture("executor-target/options.ts");
const option = { importSpecifier: "@example/db", executorTypeNames: ["QueryExecutor"] };

function optionsCase(typed) {
  return {
    code,
    filename,
    options: [option],
    languageOptions: {
      parser: tsParser,
      parserOptions: typed ? { project: resolve(dirname(filename), "tsconfig.json") } : {},
    },
  };
}

const suite = {
  invalid: [
    // accepts stable declarations whose named property has configured import identity.
    optionsCase(true),
    // requires typed services for named object declarations.
    optionsCase(false),
  ],
};

it("snapshots postgres-no-executor-adapter diagnostics", async () => {
  await expect(
    snapshotDocument(snapshotSuite("postgres-no-executor-adapter", languageOptions, suite)),
  ).toMatchFileSnapshot("snapshots/postgres-no-executor-adapter-options.snap");
}, 30_000);
