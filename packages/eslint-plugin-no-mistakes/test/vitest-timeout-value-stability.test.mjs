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
  it("uses runtime-valid object merges without admitting captured callback effects", async () => {
    assert.deepEqual(
      findings("merge-object-controls", { unknownValues: "finding" }).map(({ line, messageId }) => [
        line,
        messageId,
      ]),
      [
        [5, "timeout"],
        [6, "invalid"],
        [11, "unknown"],
      ],
    );
    assert.deepEqual(
      findings("merge-object-controls").map(({ line, messageId }) => [line, messageId]),
      [
        [5, "timeout"],
        [6, "invalid"],
      ],
    );
    await assert.doesNotReject(
      import("../../../test-cases/eslint-snippets/fixture/vitest-timeout-cap/merge-object-controls.ts"),
    );
  });
  it("keeps callback merge operands only as explicit runtime-rejection controls", async () => {
    assert.deepEqual(findings("unsupported-merge-callbacks", { unknownValues: "finding" }), []);
    await assert.rejects(
      import("../../../test-cases/eslint-snippets/fixture/vitest-timeout-cap/unsupported-merge-callbacks.ts"),
      /Cannot merge config in form of callback/,
    );
  });
  it("ignores erased type queries while retaining real callback consumers", () => {
    assert.deepEqual(
      findings("config-type-only-references", { unknownValues: "finding" }).map(
        ({ line, messageId }) => [line, messageId],
      ),
      [
        [9, "timeout"],
        [21, "unknown"],
        [21, "unknown"],
        [21, "unknown"],
        [32, "unknown"],
        [32, "unknown"],
        [32, "unknown"],
      ],
    );
    assert.deepEqual(
      findings("config-type-only-references").map(({ line, messageId }) => [line, messageId]),
      [[9, "timeout"]],
    );
  });
  it("retains unknown returned objects when identity-preserving config results escape", () => {
    const actual = findings("config-result-escapes", { unknownValues: "finding" });
    assert.deepEqual(
      [...new Set(actual.map(({ line }) => line))],
      [5, 9, 13, 19, 23, 27, 31, 35, 44],
    );
    assert.equal(actual.filter(({ messageId }) => messageId === "unknown").length, 24);
    assert.deepEqual(
      actual
        .filter(({ messageId }) => messageId !== "unknown")
        .map(({ line, messageId }) => [line, messageId]),
      [[44, "timeout"]],
    );
    assert.deepEqual(
      findings("config-result-escapes").map(({ line, messageId }) => [line, messageId]),
      [[44, "timeout"]],
    );
  });
  it("preserves returned config bindings through known consumers and immutable callback aliases", () => {
    assert.deepEqual(
      findings("config-return-values", { unknownValues: "finding" }).map(({ line, messageId }) => [
        line,
        messageId,
      ]),
      [
        [9, "timeout"],
        [13, "invalid"],
      ],
    );
  });
  it("retains unknown returned objects when callback consumers or effects are unproved", () => {
    const actual = findings("config-return-escapes", { unknownValues: "finding" });
    assert.deepEqual(
      [...new Set(actual.map(({ line }) => line))],
      [5, 10, 14, 19, 24, 28, 32, 36, 40, 45, 51, 56, 61, 65],
    );
    assert.ok(actual.every(({ messageId }) => messageId === "unknown"));
    assert.deepEqual(findings("config-return-escapes"), []);
  });
  it("keeps shallow copy writes independent while checking each copy's own stability", () => {
    assert.deepEqual(
      findings("shallow-copy-writes", { unknownValues: "finding" }).map(({ line, messageId }) => [
        line,
        messageId,
      ]),
      [
        [8, "unknown"],
        [12, "timeout"],
        [42, "unknown"],
      ],
    );
    assert.deepEqual(
      findings("shallow-copy-writes").map(({ line, messageId }) => [line, messageId]),
      [[12, "timeout"]],
    );
  });
  it("retains shared nested mutations and conservative unknown copy identities", () => {
    const actual = findings("shallow-copy-alias-mutations", { unknownValues: "finding" });
    assert.deepEqual(
      [...new Set(actual.map(({ line }) => line))],
      [4, 8, 13, 18, 26, 30, 35, 36, 40, 47, 52, 57],
    );
    assert.ok(actual.every(({ messageId }) => messageId === "unknown"));
    assert.deepEqual(findings("shallow-copy-alias-mutations"), []);
  });
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
