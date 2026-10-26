import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { beforeAll, it } from "vitest";
import { __dirname, plugin } from "./helpers.mjs";
import { realDrop12 } from "./drop12-real.mjs";

beforeAll(() => {
  realDrop12(false);
  realDrop12(true);
});
const root = resolve(
  __dirname,
  "../../../test-cases/eslint-plugin/async-call-disposition/fixture/promise-handoff-alias",
);
it.each([true, false])("projects disjoint source intersections (typed=%s)", (typed) => {
  for (const [name, lines] of [
    ["intersection-caller.ts", [5, 6, 7, 8, 9, 10, 11, 12, 13]],
    ["intersection-consumers.ts", [5]],
  ]) {
    const filename = resolve(root, name);
    const messages = new Linter({ cwd: root }).verify(
      readFileSync(filename, "utf8"),
      {
        files: ["**/*.ts"],
        languageOptions: {
          parser: tsParser,
          parserOptions: typed ? { project: resolve(root, "tsconfig.json") } : {},
        },
        plugins: { "no-mistakes": plugin },
        rules: JSON.parse(readFileSync(resolve(root, ".oxlintrc.json"), "utf8")).rules,
      },
      { filename },
    );
    assert.deepEqual(
      messages.map(({ line, messageId }) => [line, messageId]),
      lines.map((line) => [line, "disposition"]),
      name,
    );
  }
});
