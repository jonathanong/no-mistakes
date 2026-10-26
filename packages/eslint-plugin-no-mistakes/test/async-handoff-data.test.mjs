import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { beforeAll, it } from "vitest";
import { plugin } from "./helpers.mjs";
import { root, rules, cases, realGate } from "./async-handoff-data-real.mjs";

beforeAll(() => {
  realGate(true);
  realGate(false);
});
it.each([true, false])("preserves inert metadata source identities (typed=%s)", (typed) => {
  for (const [name, lines] of Object.entries(cases)) {
    const filename = resolve(root, name);
    const reports = new Linter({ cwd: root }).verify(
      readFileSync(filename, "utf8"),
      {
        files: ["**/*.ts"],
        languageOptions: {
          parser: tsParser,
          parserOptions: typed ? { project: resolve(root, "tsconfig.json") } : {},
        },
        plugins: { "no-mistakes": plugin },
        rules,
      },
      { filename },
    );
    assert.deepEqual(
      reports.map(({ line, messageId }) => [line, messageId]),
      lines.map((line) => [line, "disposition"]),
      name,
    );
  }
});
