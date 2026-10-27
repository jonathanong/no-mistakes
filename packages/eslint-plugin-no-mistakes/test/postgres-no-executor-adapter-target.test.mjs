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
const option = { importSpecifier: "@example/db", executorTypeNames: ["QueryExecutor"] };

function targetCase(name, typed = false) {
  const filename = resolve(
    __dirname,
    "../../../test-cases/eslint-snippets/fixture/executor-target",
    name,
  );
  return {
    code: fixture(`executor-target/${name}`),
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
    // accepts declared local executor slots, preserving callback reports.
    targetCase("local.ts"),
    // keeps reporting unproven, overloaded, rest and spread targets.
    targetCase("unsafe.ts"),
    // keeps executor uses inside spread containers unproven.
    targetCase("spread-containers.ts"),
    // rejects imported helpers with writes or direct eval, preserving shadowed bindings.
    targetCase("stability.ts", true),
    // requires parser type information for imported targets.
    targetCase("imported.ts"),
    targetCase("imported.ts", true),
  ],
};

it("snapshots postgres-no-executor-adapter diagnostics", async () => {
  await expect(
    snapshotDocument(snapshotSuite("postgres-no-executor-adapter", languageOptions, suite)),
  ).toMatchFileSnapshot("snapshots/postgres-no-executor-adapter-target.snap");
}, 30_000);
