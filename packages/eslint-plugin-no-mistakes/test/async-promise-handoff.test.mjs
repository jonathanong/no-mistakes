import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { it } from "vitest";
import { plugin } from "./helpers.mjs";
import {
  callerLines,
  guardLines,
  handoff,
  recallLines,
  realScenarios,
  runRealGate,
  slotLines,
} from "./async-promise-handoff-real.mjs";

const targets = JSON.parse(readFileSync(resolve(handoff, ".oxlintrc.json"), "utf8")).rules[
  "no-mistakes/async-call-disposition"
][1].targets;
function reports(name, typed) {
  const filename = resolve(handoff, name);
  return new Linter({ cwd: handoff })
    .verify(
      readFileSync(filename, "utf8"),
      {
        files: ["**/*.ts"],
        languageOptions: {
          parser: tsParser,
          parserOptions: typed
            ? {
                project: resolve(handoff, "tsconfig.json"),
                disallowAutomaticSingleRunInference: true,
              }
            : {},
        },
        plugins: { "no-mistakes": plugin },
        rules: { "no-mistakes/async-call-disposition": ["error", { targets }] },
      },
      { filename },
    )
    .map(({ line, messageId }) => [line, messageId]);
}

// Each subprocess gets its own ordinary test budget; all scenarios still run
// before ESLint, in addition to the standalone real gate used for validation.
for (const typeAware of [true, false]) {
  it.each(realScenarios.map((scenario) => [scenario.config, scenario]))(
    `proves source slots under real Oxlint (typeAware=${typeAware}, %s)`,
    (_name, scenario) => runRealGate(typeAware, [scenario]),
  );
}
it.each([true, false])("proves source target slots and preserves recall (typed=%s)", (typed) => {
  for (const [name, lines] of [
    ["caller.ts", callerLines],
    ["typed-recall.ts", recallLines],
    ["guard-caller.ts", guardLines],
    ["slot-guards.ts", slotLines],
    ["parameter-guards.ts", [5]],
    ["static-caller.ts", []],
    ["static-final.ts", [3]],
    ["extra-caller.ts", [7, 9, 11, 15, 17]],
    ["final.ts", []],
    ["dropped.ts", [4]],
  ]) {
    assert.deepEqual(
      reports(name, typed),
      lines.map((line) => [line, "disposition"]),
      name,
    );
  }
});
