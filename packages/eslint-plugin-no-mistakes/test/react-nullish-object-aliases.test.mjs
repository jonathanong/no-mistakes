import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, lint } from "./helpers.mjs";

const rule = "react-no-nullish-react-node";
describe("ReactNode prop object aliases and template destructuring", () => {
  for (const [name, lines] of [
    ["object-aliases", [7, 12, 18, 24, 30]],
    ["object-alias-loop-bindings", [15, 27, 39]],
    ["object-alias-target-shadows", [13, 23, 29, 31, 33, 43, 50, 50]],
    ["object-alias-controls", [7]],
    ["object-alias-rewrite", []],
    ["object-alias-suppressed", []],
    ["object-alias-file-suppressed", []],
  ]) {
    it(`preserves the saved ${name} decisions`, () => {
      const diagnostics = lint(
        fixture(`${rule}/${name}.tsx`),
        { [`no-mistakes/${rule}`]: "error" },
        "fixture.tsx",
      );
      assert.deepEqual(
        diagnostics.map(({ line, messageId }) => [line, messageId]),
        lines.map((line) => [line, "nullish"]),
      );
      for (const diagnostic of diagnostics) {
        assert.match(diagnostic.message, /use an explicit undefined check for fallbacks/);
      }
    });
  }
});
