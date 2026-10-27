import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { __dirname, plugin } from "./helpers.mjs";

const root = resolve(
  __dirname,
  "../../../test-cases/eslint-plugin/async-call-disposition/fixture/optional-peers/multi-hop",
);
const targets = [
  {
    sourceSpecifierPatterns: ["@app/jobs"],
    calleeNamePatterns: ["enqueueBatch"],
  },
];

it.each([
  ["caller.js", false],
  ["caller.js", true],
  ["typed-caller.ts", false],
  ["typed-caller.ts", true],
])("proves multiple imported dependency hops (%s, typed=%s)", async (file, typed) => {
  const filename = resolve(root, file);
  const messages = new Linter({ cwd: root }).verify(
    readFileSync(filename, "utf8"),
    {
      files: ["**/*.{js,ts}"],
      languageOptions: {
        parser: tsParser,
        parserOptions: typed
          ? { project: resolve(root, "tsconfig.json"), disallowAutomaticSingleRunInference: true }
          : {},
      },
      plugins: { "no-mistakes": plugin },
      rules: { "no-mistakes/async-call-disposition": ["error", { targets }] },
    },
    { filename },
  );
  assert.deepEqual(
    messages.map(({ line, messageId }) => [line, messageId]),
    (file.endsWith(".ts") ? [13, 18, 23, 28, 34] : [13, 18, 23, 28]).map((line) => [
      line,
      "disposition",
    ]),
  );
  await expect(
    `${JSON.stringify(
      messages.map(({ line, column, messageId, message }) => ({
        file: `multi-hop/${file}`,
        line,
        column,
        messageId,
        message,
      })),
      null,
      2,
    )}\n`,
  ).toMatchFileSnapshot(`snapshots/async-call-disposition-multi-hop-${file}.snap`);
});
