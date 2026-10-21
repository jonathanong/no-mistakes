import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, messages } from "./helpers.mjs";

describe("playwright same-file recall", () => {
  it("follows static keys, aliases, and forwarded Playwright calls", () => {
    assert.equal(
      messages(fixture("playwright-literals/recall.jsx"), "playwright-literals").length,
      6,
    );
    assert.equal(
      messages(
        fixture("playwright-prefer-get-by-test-id/recall.jsx"),
        "playwright-prefer-get-by-test-id",
      ).length,
      6,
    );
    assert.equal(
      messages(
        fixture("playwright-selector-priority/recall.spec.js"),
        "playwright-selector-priority",
        undefined,
        "e2e/recall.spec.js",
      ).length,
      6,
    );
    assert.equal(
      messages(
        fixture("playwright-assertion-timeout-cap/recall.spec.ts"),
        "playwright-assertion-timeout-cap",
        undefined,
        "e2e/recall.spec.ts",
      ).length,
      4,
    );
    assert.equal(
      messages(
        fixture("playwright-no-raw-scroll-pagination/recall.spec.js"),
        "playwright-no-raw-scroll-pagination",
        undefined,
        "e2e/recall.spec.js",
      ).length,
      6,
    );
    assert.equal(
      messages(
        fixture("playwright-require-exported-component-attribute/recall.tsx"),
        "playwright-require-exported-component-attribute",
        undefined,
        "recall.tsx",
      ).length,
      3,
    );
    assert.equal(
      messages(
        fixture("playwright-require-interactive-test-id/recall.jsx"),
        "playwright-require-interactive-test-id",
        { interactiveComponents: ["SaveButton"] },
      ).length,
      3,
    );
    assert.equal(
      messages(
        fixture("playwright-no-hoisted-unique-token/recall.spec.js"),
        "playwright-no-hoisted-unique-token",
        { tokenFactories: ["randomSuffix"] },
        "playwright/recall.spec.js",
      ).length,
      3,
    );
  });
});
