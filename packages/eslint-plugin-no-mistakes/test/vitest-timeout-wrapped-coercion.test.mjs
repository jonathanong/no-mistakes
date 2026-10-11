import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, lint } from "./helpers.mjs";
const findings = (name, options = {}) =>
  lint(
    fixture(`vitest-timeout-cap/${name}.ts`),
    { "no-mistakes/vitest-timeout-cap": ["error", options] },
    "fixture.test.ts",
  );
describe("wrapped copied scalar coercion", () => {
  it("preserves numeric and null updates through literal object and array wrappers", () => {
    assert.deepEqual(
      findings("wrapped-copy-coercion", { unknownValues: "finding" }).map(({ line, messageId }) => [
        line,
        messageId,
      ]),
      [
        [35, "timeout"],
        [40, "invalid"],
      ],
    );
    assert.deepEqual(
      findings("wrapped-copy-coercion").map(({ line, messageId }) => [line, messageId]),
      [
        [35, "timeout"],
        [40, "invalid"],
      ],
    );
  });
  it("rejects overwritten, shared, callable, getter, cyclic and unproved alias paths", () => {
    const actual = findings("wrapped-copy-coercion-unsafe", { unknownValues: "finding" });
    assert.deepEqual(
      [...new Set(actual.map(({ line }) => line))],
      [
        9, 15, 22, 29, 37, 39, 44, 52, 57, 63, 68, 73, 78, 83, 85, 89, 91, 98, 104, 111, 116, 121,
        126, 131, 136, 141, 146, 153, 159, 164, 168, 172, 177, 183, 190,
      ],
    );
    assert.equal(actual.length, 39);
    assert.ok(actual.every(({ messageId }) => messageId === "unknown"));
    assert.deepEqual(findings("wrapped-copy-coercion-unsafe"), []);
  });
});
