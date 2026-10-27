import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Module } from "node:module";
import { resolve } from "node:path";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { expect, it, vi } from "vitest";
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

it.each(
  [
    ["caller.js", false],
    ["caller.js", true],
    ["typed-caller.ts", false],
    ["typed-caller.ts", true],
  ].flatMap(([file, typed]) => [
    [file, typed, false],
    [file, typed, true],
  ]),
)(
  "proves multiple imported dependency hops (%s, typed=%s, typescriptOnly=%s)",
  async (file, typed, typescriptOnly) => {
    const load = Module._load;
    // Keep the host parser while exercising imported-source conversion in this
    // process too; physical absence is independently verified by the packed test.
    const peer =
      typescriptOnly &&
      vi.spyOn(Module, "_load").mockImplementation(function (name, ...args) {
        if (name === "@typescript-eslint/parser") {
          const error = new Error("Cannot find module '@typescript-eslint/parser'");
          error.code = "MODULE_NOT_FOUND";
          throw error;
        }
        return load.call(this, name, ...args);
      });
    try {
      const filename = resolve(root, file);
      const messages = new Linter({ cwd: root }).verify(
        readFileSync(filename, "utf8"),
        {
          files: ["**/*.{js,ts}"],
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
    } finally {
      if (peer) peer.mockRestore();
    }
  },
);
