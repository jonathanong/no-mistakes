import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import {
  languageOptionsFindings,
  snapshotDocument,
  snapshotSuite,
} from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

const rule = "postgres-no-executor-adapter";
const languageOptions = { ecmaVersion: 2024, sourceType: "module", globals: {} };
const config = {
  importSpecifier: "@example/db",
  executorTypeNames: ["QueryExecutor"],
  executorNames: ["read"],
};
function sample(name, typed, mode) {
  const filename = resolve(
    __dirname,
    "../../../test-cases/eslint-snippets/fixture/executor-target",
    name,
  );
  return {
    code: fixture(`executor-target/${name}`),
    filename,
    options: [{ ...config, mode }],
    languageOptions: {
      parser: tsParser,
      parserOptions: typed ? { projectService: true, tsconfigRootDir: dirname(filename) } : {},
    },
  };
}

it.each(["all", "forwarding-only"])(
  "proves configured namespace unions and rejects unconfigured branches (%s)",
  async (mode) => {
    const invalid = [
      ["namespace-union-local.ts", false, [20, 27, 28, 29, 30, 32, 33, 34, 40]],
      ["namespace-union-local.ts", true, [20, 27, 28, 29, 30, 32, 33, 34, 40]],
      ["namespace-union-imported.ts", true, [8, 9, 10, 11, 13, 14, 15, 16, 19]],
      ["namespace-union-imported.ts", false, Array.from({ length: 16 }, (_, i) => i + 4)],
    ].map(([name, typed, expected]) => {
      const item = sample(name, typed, mode);
      expect(languageOptionsFindings({ ...item, rule }).map(({ line }) => line)).toEqual(expected);
      return item;
    });
    await expect(
      snapshotDocument(snapshotSuite(rule, languageOptions, { invalid })),
    ).toMatchFileSnapshot(`snapshots/postgres-no-executor-adapter-namespace-union-${mode}.snap`);
  },
);
