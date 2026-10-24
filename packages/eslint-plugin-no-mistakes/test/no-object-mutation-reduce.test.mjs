import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, lint } from "./helpers.mjs";

const rules = { "no-mistakes/no-object-mutation": "error" };

function reports(name) {
  const code = fixture(name);
  const expected = code
    .split("\n")
    .flatMap((line, index) => (line.includes("// report:") ? [index + 1] : []));
  const actual = lint(code, rules, "fixture.ts");
  assert.deepEqual(
    actual.map((message) => message.messageId),
    expected.map(() => "mutation"),
  );
  assert.deepEqual(
    actual.map((message) => message.line),
    expected,
  );
}

describe("no-object-mutation reducer ownership", () => {
  it("owns literal accumulators while preserving nested and escaped provenance", () => {
    reports("no-object-mutation-reduce.ts");
  });
  it("keeps shared seeds and callbacks without a stable accumulator conservative", () => {
    reports("no-object-mutation-reduce-controls.ts");
  });
});
