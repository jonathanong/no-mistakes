import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { lint } from "./helpers.mjs";

const source = "const ready = true;\n";

const cases = [
  {
    rule: "playwright-assertion-timeout-cap",
    invalid: { max: "slow" },
    valid: { max: 1000 },
  },
  {
    rule: "playwright-consistent-attribute",
    invalid: { canonicalAttribute: 1 },
    valid: { canonicalAttribute: "data-testid" },
  },
  {
    rule: "playwright-defaults",
    invalid: { selectorAttributes: "data-testid" },
    valid: { selectorAttributes: ["data-testid"] },
  },
  {
    rule: "playwright-literals",
    invalid: { allowDefaultedProps: "yes" },
    valid: { allowDefaultedProps: true },
  },
  {
    rule: "playwright-naming-convention",
    invalid: { pattern: 1 },
    valid: { pattern: "^[a-z]+$" },
  },
  {
    rule: "playwright-no-empty",
    invalid: { selectorAttributes: "data-testid" },
    valid: { selectorAttributes: ["data-testid"] },
  },
  {
    rule: "playwright-no-hoisted-unique-token",
    invalid: { tokenFactories: "uniqueToken" },
    valid: { tokenFactories: ["uniqueToken"] },
  },
  {
    rule: "playwright-no-raw-scroll-pagination",
    invalid: { cursorParams: "after" },
    valid: { cursorParams: ["after"] },
  },
  {
    rule: "playwright-no-swallowed-visibility",
    invalid: { methods: "isVisible" },
    valid: { methods: ["isVisible"] },
  },
  {
    rule: "playwright-prefer-get-by-test-id",
    invalid: { selectorAttributes: "data-testid" },
    valid: { selectorAttributes: ["data-testid"] },
  },
  {
    rule: "playwright-require-exported-component-attribute",
    invalid: { attributes: "data-pw" },
    valid: { attributes: ["data-pw"] },
  },
  {
    rule: "playwright-require-interactive-test-id",
    invalid: { slotProps: "asChild" },
    valid: { slotProps: ["asChild"] },
  },
  {
    rule: "playwright-unique",
    invalid: { selectorAttributes: "data-testid" },
    valid: { selectorAttributes: ["data-testid"] },
  },
];

function lintRule(rule, option) {
  return lint(source, { [`no-mistakes/${rule}`]: ["error", option] }, "fixture.tsx");
}

describe("playwright rules reject invalid options", () => {
  for (const { rule, invalid, valid } of cases) {
    it(`${rule} rejects one invalid option and still runs with a valid option`, () => {
      assert.throws(() => lintRule(rule, invalid));
      assert.ok(Array.isArray(lintRule(rule, valid)));
    });
  }
});
