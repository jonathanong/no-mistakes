import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, messages } from "./helpers.mjs";

function fixture(name) {
  return readFileSync(
    resolve(__dirname, "../../../fixtures/eslint-plugin/upstreamed-generic", name),
    "utf8",
  );
}

describe("upstreamed generic rule coverage", () => {
  it("covers additional branches in upstreamed rules", () => {
    const code = fixture("coverage.tsx");
    assert.deepEqual(messages(code, "await-array-methods", undefined, "coverage.tsx"), []);
    assert.deepEqual(messages(code, "no-delete-property", undefined, "coverage.tsx"), [
      "delete",
      "delete",
    ]);
    assert.deepEqual(
      messages(code, "nextjs-metadata-exports-location", undefined, "app/components/meta.tsx"),
      ["location"],
    );
    assert.deepEqual(messages(code, "test-no-error-message-matching", undefined, "coverage.tsx"), [
      "message",
      "message",
      "message",
    ]);
    assert.deepEqual(
      messages(
        fixture("message-angle.ts"),
        "test-no-error-message-matching",
        undefined,
        "message-angle.ts",
      ),
      ["message"],
    );
    assert.deepEqual(messages(code, "test-no-shared-state", undefined, "coverage.tsx"), [
      "shared",
      "shared",
      "shared",
      "shared",
      "shared",
      "shared",
      "shared",
      "shared",
      "shared",
    ]);
    assert.deepEqual(messages(code, "no-vitest-sequential", undefined, "coverage.tsx"), [
      "sequential",
    ]);
    assert.deepEqual(messages(code, "playwright-selector-priority", undefined, "e2e.spec.tsx"), [
      "semantic",
    ]);
    assert.deepEqual(
      messages(code, "playwright-assertion-timeout-cap", undefined, "e2e.spec.tsx"),
      ["timeout", "timeout", "timeout"],
    );
    assert.deepEqual(
      messages(code, "playwright-assertion-timeout-cap", { max: 20000 }, "e2e.spec.tsx"),
      ["timeout"],
    );
    assert.deepEqual(
      messages(code, "playwright-assertion-timeout-cap", undefined, "unit.test.tsx"),
      [],
    );
    assert.deepEqual(
      messages(code, "playwright-selector-priority", undefined, "unit.test.tsx"),
      [],
    );
  });
});
