import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";
import { it } from "vitest";
import { __dirname, plugin } from "./helpers.mjs";

const root = resolve(
  __dirname,
  "../../../test-cases/eslint-plugin/postgres-manual-transaction-recall/fixture",
);
const rule = "no-mistakes/postgres-no-manual-transaction";
const cases = [
  ["pg", "pg", [4, 5, 8, 10, 12, 15, 18]],
  ["configured", "@data-stores/psql", [3, 4, 7, 9, 11, 14, 17]],
];
function lint(name, typed, options) {
  const filename = resolve(root, `${name}.ts`);
  return new Linter({ cwd: root })
    .verify(
      readFileSync(filename, "utf8"),
      {
        files: ["**/*.ts"],
        languageOptions: {
          parser: tsParser,
          parserOptions: typed
            ? { project: resolve(root, "tsconfig.json"), disallowAutomaticSingleRunInference: true }
            : {},
        },
        plugins: { "no-mistakes": plugin },
        rules: { [rule]: ["error", options] },
      },
      { filename },
    )
    .map(({ line, messageId }) => [line, messageId]);
}
it.each([false, true])(
  "retains manual transaction recall through connect wrappers (typed=%s)",
  (typed) => {
    for (const [name, importSpecifier, lines] of cases) {
      assert.deepEqual(
        lint(name, typed, { importSpecifier }),
        lines.map((line) => [line, "manualTransaction"]),
      );
      assert.deepEqual(lint(name, typed, { importSpecifier, executorNames: [] }), []);
      assert.deepEqual(lint(name, typed, { importSpecifier, executorNames: ["execute"] }), []);
    }
    assert.deepEqual(lint("controls", typed, { importSpecifier: "@data-stores/psql" }), []);
  },
);
it("matches recall and suppression in real type-aware Oxlint", () => {
  for (const [name, , lines] of cases) {
    const result = spawnSync(
      process.execPath,
      [
        resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
        "--type-aware",
        "--deny-warnings",
        "--format",
        "json",
        "--config",
        resolve(root, `.oxlintrc-${name}.json`),
        resolve(root, `${name}.ts`),
      ],
      { encoding: "utf8" },
    );
    assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
    assert.deepEqual(
      JSON.parse(result.stdout).diagnostics.map(({ labels, code }) => [labels[0].span.line, code]),
      lines.map((line) => [line, "no-mistakes(postgres-no-manual-transaction)"]),
    );
  }
});
