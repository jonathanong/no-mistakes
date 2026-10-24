import assert from "node:assert/strict";
import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";
import { describe, it, vi } from "vitest";
import { __dirname, fixture, plugin } from "./helpers.mjs";

function findings(
  typed,
  rule = "postgres-no-executor-adapter",
  name = "options-escapes.ts",
  parser = tsParser,
) {
  const filename = resolve(
    __dirname,
    `../../../test-cases/eslint-snippets/fixture/executor-target/${name}`,
  );
  return new Linter({ cwd: dirname(filename) })
    .verify(
      fixture(`executor-target/${name}`),
      {
        files: ["**/*.ts"],
        languageOptions: {
          parser,
          // Repeated rule assertions need the project, rather than autofix fallback.
          parserOptions: typed
            ? {
                project: resolve(dirname(filename), "tsconfig.json"),
                disallowAutomaticSingleRunInference: true,
              }
            : {},
        },
        plugins: { "no-mistakes": plugin },
        rules: {
          [`no-mistakes/${rule}`]: [
            "error",
            { importSpecifier: "@example/db", executorTypeNames: ["QueryExecutor"] },
          ],
        },
      },
      { filename },
    )
    .map(({ line, messageId }) => `${line}:${messageId}`);
}

describe("executor option helper escapes", () => {
  it("retains diagnostics when repeated single-run parsing loses module resolution", () => {
    vi.stubEnv("TSESTREE_SINGLE_RUN", "true");
    try {
      assert.deepEqual(findings(true, undefined, "isolated-options.ts"), []);
      assert.deepEqual(findings(true, undefined, "isolated-options.ts"), [
        "9:passed",
        "10:passed",
        "12:passed",
      ]);
    } finally {
      vi.unstubAllEnvs();
    }
  });

  it("keeps reporting when the parser cannot map the implementation body", () => {
    const parser = {
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
    assert.deepEqual(findings(true, undefined, "options-missing-body.ts", parser), ["5:passed"]);
  });

  it("retains incoming diagnostics when a local helper exposes its executor", () => {
    assert.deepEqual(
      findings(true),
      [
        23, 25, 35, 37, 38, 39, 40, 41, 42, 43, 47, 49, 50, 51, 52, 53, 54, 55, 76, 77, 78, 80, 81,
        82, 86, 88, 105, 106, 107, 109, 110, 111, 114, 115, 122, 123, 144, 145, 161, 162, 163, 167,
        168, 169, 179, 182, 183, 192, 193, 194, 196, 201, 204, 208, 209, 210, 211,
      ].map((line) => `${line}:passed`),
    );
  });

  it("keeps the syntax-only inline option escape diagnostic", () => {
    assert.ok(findings(false).includes("47:passed"));
    assert.ok(!findings(false).includes("120:passed"));
  });

  it("shows direct value aliases are judged while object aliases are not", () => {
    assert.deepEqual(findings(true, "postgres-no-manual-transaction"), [
      "103:manualTransaction",
      "152:manualTransaction",
      "153:manualTransaction",
      "154:manualTransaction",
      "170:manualTransaction",
      "171:manualTransaction",
      "172:manualTransaction",
      "185:manualTransaction",
      "213:manualTransaction",
      "214:manualTransaction",
      "215:manualTransaction",
      "218:manualTransaction",
    ]);
    assert.deepEqual(
      findings(true, "postgres-no-manual-transaction", "options-escape-helpers.ts"),
      [
        "64:manualTransaction",
        "69:manualTransaction",
        "70:manualTransaction",
        "71:manualTransaction",
        "84:manualTransaction",
        "85:manualTransaction",
        "86:manualTransaction",
      ],
    );
  });

  it("does not exempt options captured by direct eval", () => {
    assert.deepEqual(findings(true, undefined, "options-eval.ts"), ["7:passed", "8:passed"]);
    assert.deepEqual(findings(false, undefined, "options-eval.ts"), ["7:passed", "8:passed"]);
  });

  it("does not exempt options captured by eval through erased assertions", () => {
    assert.deepEqual(findings(true, undefined, "options-wrapped-eval.ts"), [
      "7:passed",
      "8:passed",
    ]);
    assert.deepEqual(findings(false, undefined, "options-wrapped-eval.ts"), [
      "7:passed",
      "8:passed",
    ]);
  });

  it("keeps await and yield callees unproven when their operand is non-null asserted", () => {
    assert.deepEqual(findings(true, undefined, "options-await-assertions.ts"), [
      "8:passed",
      "9:passed",
      "10:passed",
      "11:passed",
    ]);
    assert.deepEqual(
      findings(true, "postgres-no-manual-transaction", "options-await-assertions.ts"),
      [],
    );
  });
});
