import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { expect, it, vi } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const option = { importSpecifier: "@example/db", executorTypeNames: ["QueryExecutor"] };

function escapeCase(name, typed, parser = tsParser) {
  const filename = resolve(
    __dirname,
    `../../../test-cases/eslint-snippets/fixture/executor-target/${name}`,
  );
  return {
    code: fixture(`executor-target/${name}`),
    filename,
    options: [option],
    languageOptions: {
      parser,
      parserOptions: typed
        ? {
            project: resolve(dirname(filename), "tsconfig.json"),
            disallowAutomaticSingleRunInference: true,
          }
        : {},
    },
  };
}

it("snapshots postgres executor option escape diagnostics", async () => {
  const missingBodyParser = {
    ...tsParser,
    parseForESLint(...args) {
      const parsed = tsParser.parseForESLint(...args);
      const map = parsed.services.esTreeNodeToTSNodeMap;
      parsed.services.esTreeNodeToTSNodeMap = {
        get(node) {
          return node.type === "FunctionDeclaration" ? undefined : map.get(node);
        },
      };
      return parsed;
    },
  };
  vi.stubEnv("TSESTREE_SINGLE_RUN", "true");
  let singleRun;
  try {
    singleRun = snapshotSuite("postgres-no-executor-adapter", languageOptions, {
      valid: [
        // retains diagnostics when repeated single-run parsing loses module resolution.
        // Zero findings.
        escapeCase("isolated-options.ts", true),
      ],
      invalid: [
        // retains diagnostics when repeated single-run parsing loses module resolution.
        // Three passed findings.
        escapeCase("isolated-options.ts", true),
      ],
    });
  } finally {
    vi.unstubAllEnvs();
  }
  await expect(
    snapshotDocument({
      "postgres-no-executor-adapter single-run": singleRun,
      "postgres-no-executor-adapter": snapshotSuite(
        "postgres-no-executor-adapter",
        languageOptions,
        {
          invalid: [
            // keeps reporting when the parser cannot map the implementation body.
            // One passed finding.
            escapeCase("options-missing-body.ts", true, missingBodyParser),
            // retains incoming diagnostics when a local helper exposes its executor.
            escapeCase("options-escapes.ts", true),
            // keeps the syntax-only inline option escape diagnostic.
            escapeCase("options-escapes.ts", false),
            // does not exempt options captured by direct eval.
            escapeCase("options-eval.ts", true),
            escapeCase("options-eval.ts", false),
            // does not exempt options captured by eval through erased assertions.
            escapeCase("options-wrapped-eval.ts", true),
            escapeCase("options-wrapped-eval.ts", false),
            // keeps await and yield callees unproven when their operand is non-null asserted.
            // Four passed findings.
            escapeCase("options-await-assertions.ts", true),
          ],
        },
      ),
      "postgres-no-manual-transaction": snapshotSuite(
        "postgres-no-manual-transaction",
        languageOptions,
        {
          valid: [
            // keeps await and yield callees unproven when their operand is non-null asserted.
            // Zero findings.
            escapeCase("options-await-assertions.ts", true),
          ],
          invalid: [
            // shows direct value aliases are judged while object aliases are not.
            escapeCase("options-escapes.ts", true),
            escapeCase("options-escape-helpers.ts", true),
          ],
        },
      ),
    }),
  ).toMatchFileSnapshot("snapshots/postgres-no-executor-adapter-options-escapes.snap");
}, 30_000);
