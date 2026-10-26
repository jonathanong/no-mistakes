import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { Module } from "node:module";
import { resolve } from "node:path";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { it, vi } from "vitest";
import { plugin } from "./helpers.mjs";
import { fixture, realScenarios } from "./async-promise-handoff-real.mjs";
import {
  root as dataRoot,
  cases as dataCases,
  rules as dataRules,
} from "./async-handoff-data-real.mjs";

const scenarios = realScenarios.map((scenario) => ({
  name: scenario.config,
  root: fixture,
  rules: JSON.parse(readFileSync(resolve(fixture, scenario.config), "utf8")).rules,
  files: Object.fromEntries(
    (scenario.files ?? Object.keys(scenario.lines)).map((file) => [
      file,
      scenario.lines[file] ?? [],
    ]),
  ),
}));
scenarios.push({ name: "handoff-data", root: dataRoot, rules: dataRules, files: dataCases });
scenarios.push({
  name: "optional-peer-syntax",
  root: resolve(fixture, "optional-peers"),
  rules: JSON.parse(readFileSync(resolve(fixture, "optional-peers/.oxlintrc.json"), "utf8")).rules,
  files: {
    "syntax.js": [],
    "heritage.js": [],
    "type-shadow.js": [4, 6, 8, 12],
    "recall.js": [3, 5, 7, 9, 12, 15],
    "unsupported.js": [3],
  },
});
it.each(scenarios)("keeps source proof and recall with only TypeScript ($name)", (scenario) => {
  const original = Module._load;
  const unavailable = vi.spyOn(Module, "_load").mockImplementation(function (name, ...args) {
    if (name === "@typescript-eslint/parser") {
      throw Object.assign(new Error("Cannot find module '@typescript-eslint/parser'"), {
        code: "MODULE_NOT_FOUND",
      });
    }
    return original.call(this, name, ...args);
  });
  try {
    for (const [file, lines] of Object.entries(scenario.files)) {
      const filename = resolve(scenario.root, file);
      const messages = new Linter({ cwd: scenario.root }).verify(
        readFileSync(filename, "utf8"),
        {
          files: ["**/*.{js,ts}"],
          languageOptions: { parser: tsParser },
          plugins: { "no-mistakes": plugin },
          rules: {
            "no-mistakes/async-call-disposition":
              scenario.rules["no-mistakes/async-call-disposition"],
          },
        },
        { filename },
      );
      assert.deepEqual(
        messages.map(({ line, messageId }) => [line, messageId]),
        lines.map((line) => [line, "disposition"]),
        file,
      );
    }
  } finally {
    unavailable.mockRestore();
  }
});
