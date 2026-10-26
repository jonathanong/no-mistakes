import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Module, createRequire } from "node:module";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";
import { it, vi } from "vitest";
import { fixture } from "./async-promise-handoff-real.mjs";
import { plugin } from "./helpers.mjs";
const root = resolve(fixture, "optional-peers");
it.each([false, true])(
  "erased types cannot hide sensitive globals (TypeScript-only=%s)",
  (missingParser) => {
    const original = Module._load;
    const load = vi.spyOn(Module, "_load").mockImplementation(function (name, ...args) {
      if (missingParser && name === "@typescript-eslint/parser") {
        throw Object.assign(new Error("Cannot find module '@typescript-eslint/parser'"), {
          code: "MODULE_NOT_FOUND",
        });
      }
      return original.call(this, name, ...args);
    });
    try {
      for (const [file, line] of [
        ["global-type.js", 3],
        ["global-type-import.js", 3],
        ["global-type-specifier.js", 3],
        ["global-type-local.ts", 9],
        ["global-value.js", null],
      ]) {
        const filename = resolve(root, file);
        const messages = new Linter({ cwd: root }).verify(
          readFileSync(filename, "utf8"),
          {
            files: ["**/*.{js,ts}"],
            languageOptions: { parser: tsParser },
            plugins: { "no-mistakes": plugin },
            rules: JSON.parse(readFileSync(resolve(root, ".oxlintrc.json"), "utf8")).rules,
          },
          { filename },
        );
        assert.deepEqual(
          messages.map(({ line, messageId }) => [line, messageId]),
          line === null ? [] : [[line, "disposition"]],
          file,
        );
      }
    } finally {
      load.mockRestore();
    }
  },
);

it("observes global writes behind erased declarations", () => {
  const { globalObserverMutated } = createRequire(import.meta.url)(
    "../src/rules/async-observer-mutations",
  );
  const filename = resolve(root, "erased-globals.ts");
  let observed;
  const messages = new Linter({ cwd: root }).verify(
    readFileSync(filename, "utf8"),
    {
      files: ["**/*.ts"],
      languageOptions: { parser: tsParser },
      plugins: {
        facts: {
          rules: {
            capture: {
              create(context) {
                return {
                  "Program:exit"() {
                    observed = {
                      Object: globalObserverMutated("Object", context),
                      Map: globalObserverMutated("Map", context),
                    };
                  },
                };
              },
            },
          },
        },
      },
      rules: { "facts/capture": "error" },
    },
    { filename },
  );
  assert.deepEqual(messages, []);
  assert.deepEqual(observed, { Object: true, Map: true });
});
