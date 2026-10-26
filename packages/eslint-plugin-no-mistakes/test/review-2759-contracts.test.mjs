import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { it } from "vitest";
import { __dirname, plugin } from "./helpers.mjs";

const repo = resolve(__dirname, "../../..");
const cases = [
  ["test-cases/eslint-plugin/async-call-disposition/fixture/optional-peers", "configured.js", []],
  // Unknown receiver identity must not hide explicit manual transaction commands.
  ["test-cases/eslint-snippets/fixture/postgres-2755", "constructed-clients.ts", [8, 10, 11, 12]],
];
it.each(cases)(
  "checks declared contracts under real type-aware Oxlint (%s)",
  (path, file, lines) => {
    const root = resolve(repo, path);
    const result = spawnSync(
      process.execPath,
      [
        resolve(repo, "node_modules/oxlint/bin/oxlint"),
        "--type-aware",
        "--deny-warnings",
        "--config",
        resolve(root, file === "configured.js" ? ".oxlintrc-contracts.json" : ".oxlintrc.json"),
        "--format",
        "json",
        resolve(root, file),
      ],
      { cwd: root, encoding: "utf8" },
    );
    assert.equal(result.status, lines.length ? 1 : 0, result.stderr);
    assert.deepEqual(
      JSON.parse(result.stdout).diagnostics.map(({ labels, code }) => [labels[0].span.line, code]),
      lines.map((line) => [line, "no-mistakes(postgres-no-manual-transaction)"]),
    );
  },
);
for (const [path, file, lines] of cases) {
  it.each([false, true])(`checks declared contracts in ESLint (${file}, typed=%s)`, (typed) => {
    const root = resolve(repo, path);
    const filename = resolve(root, file);
    const rules = JSON.parse(
      readFileSync(
        resolve(root, file === "configured.js" ? ".oxlintrc-contracts.json" : ".oxlintrc.json"),
        "utf8",
      ),
    ).rules;
    const messages = new Linter({ cwd: root }).verify(
      readFileSync(filename, "utf8"),
      {
        files: ["**/*.{ts,js}"],
        languageOptions: {
          parser: tsParser,
          parserOptions: typed
            ? {
                project: resolve(root, "tsconfig.json"),
                disallowAutomaticSingleRunInference: true,
              }
            : {},
        },
        plugins: { "no-mistakes": plugin },
        rules,
      },
      { filename },
    );
    assert.deepEqual(
      messages.map(({ line, messageId, ruleId }) => [line, messageId, ruleId]),
      lines.map((line) => [
        line,
        "manualTransaction",
        "no-mistakes/postgres-no-manual-transaction",
      ]),
    );
  });
}
