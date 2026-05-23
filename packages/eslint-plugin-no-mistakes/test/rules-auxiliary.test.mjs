import assert from "node:assert/strict";
import { performance } from "node:perf_hooks";
import { describe, it } from "vitest";
import { fixture, lint, messages, plugin, require } from "./helpers.mjs";

const { cssSelectorValues } = require("../src/helpers");

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

  it("does not backtrack catastrophically on malformed CSS attribute selectors", () => {
    const source = "[data-pw=save" + " ".repeat(5000);
    const start = performance.now();
    const values = cssSelectorValues(source, ["data-pw"]);
    const elapsed = performance.now() - start;

    assert.deepEqual(values, []);
    assert.ok(elapsed < 1000, `selector parsing took ${elapsed}ms`);
  }, 5000);
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
