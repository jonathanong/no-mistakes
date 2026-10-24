import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, lint } from "./helpers.mjs";

const findings = (name, options = {}) =>
  lint(
    fixture(`vitest-timeout-cap/${name}.ts`),
    { "no-mistakes/vitest-timeout-cap": ["error", options] },
    "fixture.test.ts",
  ).map(({ line, messageId }) => [line, messageId]);

describe("vitest-timeout-cap documented limits", () => {
  it("keeps helper parameters unknown without substituting caller arguments", () => {
    assert.deepEqual(findings("limits-parameter"), []);
    assert.deepEqual(findings("limits-parameter", { unknownValues: "finding" }), [[5, "unknown"]]);
  });

  it("does not admit a default import or another file's runner re-export", () => {
    assert.deepEqual(findings("limits-runner-identity"), []);
    assert.deepEqual(findings("limits-runner-identity", { unknownValues: "finding" }), []);
  });

  it("resolves the immutable budget rewrite against the configured cap", () => {
    assert.deepEqual(findings("limits-budget-rewrite"), []);
    assert.deepEqual(findings("limits-budget-rewrite", { unknownValues: "finding" }), []);
    assert.deepEqual(findings("limits-budget-rewrite", { overrideMax: 3_000 }), [[5, "timeout"]]);
  });
});
