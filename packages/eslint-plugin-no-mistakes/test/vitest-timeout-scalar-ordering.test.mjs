import assert from "node:assert/strict";
import { it } from "vitest";
import { fixture, lint } from "./helpers.mjs";

it("preserves earlier scalar coercion without admitting uncertain write ordering", () => {
  const findings = (options) =>
    lint(
      fixture("vitest-timeout-cap/copy-scalar-ordering.ts"),
      { "no-mistakes/vitest-timeout-cap": ["error", options] },
      "fixture.test.ts",
    ).map(({ line, messageId }) => [line, messageId]);
  assert.deepEqual(findings({ unknownValues: "finding" }), [
    [8, "timeout"],
    ...[19, 24, 30, 35, 39, 45, 49, 54, 59, 65].flatMap((line) => Array(3).fill([line, "unknown"])),
    [76, "unknown"],
  ]);
  assert.deepEqual(findings({}), [[8, "timeout"]]);
});
