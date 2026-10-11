import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, lint } from "./helpers.mjs";
const findings = (name, options = {}) =>
  lint(
    fixture(`vitest-timeout-cap/${name}.ts`),
    { "no-mistakes/vitest-timeout-cap": ["error", options] },
    "fixture.test.ts",
  );
describe("vitest-timeout-cap value stability", () => {
  it("retains unknown deadlines after member writes, aliases, deletion and helper escape", () => {
    const strict = { unknownValues: "finding", defaultMax: 30000 };
    const actual = findings("mutable-values", strict);
    assert.deepEqual([...new Set(actual.map(({ line }) => line))], [5, 9, 12, 15, 16, 22]);
    assert.ok(actual.every(({ messageId }) => messageId === "unknown"));
    assert.ok(actual.some(({ line, message }) => line === 16 && message.includes("testTimeout")));
    assert.ok(actual.some(({ line, message }) => line === 22 && message.includes("testTimeout")));
    assert.deepEqual(findings("mutable-values"), []);
  });
  it("preserves immutable aliases, scalar budgets and explicit restoration", () => {
    assert.deepEqual(
      findings("immutable-value-controls", {
        unknownValues: "finding",
        defaultMax: 30000,
      }),
      [],
    );
  });
  it("caps test and hook overrides on registrars built through supported Vitest fixture APIs", () => {
    assert.deepEqual(
      findings("supported-fixture-builders", {
        unknownValues: "finding",
      }).map(({ line, messageId }) => [line, messageId]),
      [
        [3, "timeout"],
        [4, "invalid"],
        [6, "timeout"],
        [8, "invalid"],
      ],
    );
  });
  it("retains unknown object exports including declaration, named alias and default", () => {
    const actual = findings("exported-object-values", { unknownValues: "finding" });
    assert.deepEqual([...new Set(actual.map(({ line }) => line))], [3, 7, 10]);
    assert.ok(actual.every(({ messageId }) => messageId === "unknown"));
  });
  it("preserves exported scalar budgets without hiding an oversized scalar", () => {
    assert.deepEqual(
      findings("exported-scalar-controls", {
        unknownValues: "finding",
      }).map(({ line, messageId }) => [line, messageId]),
      [[10, "timeout"]],
    );
  });
  it("rejects unproved yield, await, tagged-template and computed-key consumers", () => {
    const actual = findings("unproved-value-escapes", { unknownValues: "finding" });
    assert.deepEqual([...new Set(actual.map(({ line }) => line))], [4, 7, 10, 13]);
    assert.ok(actual.every(({ messageId }) => messageId === "unknown"));
  });
});
