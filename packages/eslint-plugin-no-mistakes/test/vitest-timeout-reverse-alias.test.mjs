import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, lint } from "./helpers.mjs";

const findings = (name, options = {}) =>
  lint(
    fixture(`vitest-timeout-cap/${name}.ts`),
    { "no-mistakes/vitest-timeout-cap": ["error", options] },
    "fixture.test.ts",
  ).map(({ line, messageId }) => [line, messageId]);

describe("vitest-timeout-cap reverse copied aliases", () => {
  it("checks original, forward, sibling and chained writes without losing scalar controls", () => {
    assert.deepEqual(
      findings("copy-reverse-alias-overwrites", { unknownValues: "finding" }),
      [4, 12, 19, 28, 58, 65, 73, 80, 87, 93, 100, 107].flatMap((line) =>
        Array(3).fill([line, "unknown"]),
      ),
    );
    assert.deepEqual(findings("copy-reverse-alias-overwrites"), []);
  });

  it("retains unknown shared coercion and verifies its actual mutation of both sources", async () => {
    assert.deepEqual(
      findings("copy-reverse-alias-source-mutation", { unknownValues: "finding" }),
      [4, 10, 10, 17, 23, 23].map((line) => [line, "unknown"]),
    );
    assert.deepEqual(findings("copy-reverse-alias-source-mutation"), []);
    const { observedTimeouts } =
      await import("../../../test-cases/eslint-snippets/fixture/vitest-timeout-cap/copy-reverse-alias-source-mutation.ts");
    assert.deepEqual(observedTimeouts, [30_001, 60_000]);
  });

  it("preserves consumer and ordering guards through every alias direction", () => {
    assert.deepEqual(
      findings("copy-alias-guard-composition", { unknownValues: "finding" }),
      [10, 17, 24, 24, 28, 28, 28, 28, 29, 29].map((line) => [line, "unknown"]),
    );
    assert.deepEqual(findings("copy-alias-guard-composition"), []);
  });

  it("rejects an absent first alias initializer without losing the ordinary alias proof", () => {
    assert.deepEqual(
      findings("copy-alias-ambient", { unknownValues: "finding" }),
      [8, 17].map((line) => [line, "unknown"]),
    );
    assert.deepEqual(findings("copy-alias-ambient"), []);
  });
});
