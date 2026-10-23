import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, lint } from "./helpers.mjs";

const rule = "ts-no-function-aliases";
// Each source is saved so type identity, shadowing, and receiver counterexamples
// stay reviewable independently of the implementation's normalization helpers.
describe("cast identity and reachability safety corpus", () => {
  for (const { name, code, expected } of JSON.parse(
    fixture("ts-object-recall/cast-safety-corpus.json"),
  )) {
    it(name, () => {
      const diagnostics = lint(code, { [`no-mistakes/${rule}`]: "error" }, "fixture.ts");
      assert.equal(
        diagnostics.some((item) => item.fatal),
        false,
      );
      assert.deepEqual(
        diagnostics.map((item) => item.messageId),
        Array.from({ length: expected }, () => "alias"),
      );
    });
  }
});
