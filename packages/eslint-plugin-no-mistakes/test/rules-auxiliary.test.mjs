import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, lint, messages, plugin } from "./helpers.mjs";

describe("playwright-no-empty", () => {
  it("reports empty literal test IDs", () => {
    assert.deepEqual(
      messages(
        "<><button data-pw='' /><button data-pw /><button data-pw={'ok'} /></>;",
        "playwright-no-empty",
      ),
      ["empty"],
    );
  });
});

describe("playwright-prefer-get-by-test-id", () => {
  it("reports exact CSS test-id selectors in Playwright selector calls", () => {
    assert.deepEqual(
      messages(fixture("prefer-get-by-testid.js"), "playwright-prefer-get-by-test-id"),
      ["prefer", "prefer", "prefer", "prefer", "prefer", "prefer", "prefer"],
    );
  });
});

describe("playwright-naming-convention", () => {
  it("checks literal values against a configurable pattern", () => {
    assert.deepEqual(
      messages(
        "<><button data-pw='SaveButton' /><button data-pw='save-button' /></>;",
        "playwright-naming-convention",
      ),
      ["naming"],
    );
    assert.deepEqual(
      messages("<button data-pw='SaveButton' />;", "playwright-naming-convention", {
        pattern: "^[A-Z][A-Za-z]+$",
      }),
      [],
    );
  });
});

describe("strict config", () => {
  it("runs the strict rule set", () => {
    const messages = lint("<button data-testid='Save' />;", plugin.configs.strict.rules);
    assert.deepEqual(messages.map((message) => message.ruleId).sort(), [
      "no-mistakes/playwright-consistent-attribute",
      "no-mistakes/playwright-naming-convention",
    ]);
  });
});
