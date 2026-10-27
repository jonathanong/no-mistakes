import assert from "node:assert/strict";
import { performance } from "node:perf_hooks";
import { describe, expect, it } from "vitest";
import {
  configuredFindings,
  pluginFindings,
  snapshotCases,
  snapshotDocument,
} from "./diagnostic-snapshot.mjs";
import { fixture, plugin, require } from "./helpers.mjs";

const { cssSelectorValues } = require("../src/helpers");

function lintCase(name, code, rule, option) {
  return {
    name,
    rule,
    ...(option !== undefined ? { options: option } : {}),
    findings: pluginFindings(code, rule, option),
  };
}

it("snapshots rules-auxiliary diagnostics", async () => {
  const uppercase = { pattern: "^[A-Z][A-Za-z]+$" };
  const cases = [
    lintCase(
      "reports empty literal test IDs",
      "<><button data-pw='' /><button data-pw /><button data-pw={'ok'} /></>;",
      "playwright-no-empty",
    ),
    lintCase(
      "reports exact CSS test-id selectors in Playwright selector calls",
      fixture("prefer-get-by-testid.js"),
      "playwright-prefer-get-by-test-id",
    ),
    lintCase(
      "checks literal values against a configurable pattern: default",
      "<><button data-pw='SaveButton' /><button data-pw='save-button' /></>;",
      "playwright-naming-convention",
    ),
    lintCase(
      "checks literal values against a configurable pattern: uppercase",
      "<button data-pw='SaveButton' />;",
      "playwright-naming-convention",
      uppercase,
    ),
    {
      name: "runs the strict rule set",
      findings: configuredFindings("<button data-testid='Save' />;", plugin.configs.strict.rules),
    },
  ];
  await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
    "snapshots/rules-auxiliary.snap",
  );
});

describe("playwright-prefer-get-by-test-id", () => {
  it("does not backtrack catastrophically on malformed CSS attribute selectors", () => {
    const source = "[data-pw=save" + " ".repeat(5000);
    const start = performance.now();
    const values = cssSelectorValues(source, ["data-pw"]);
    const elapsed = performance.now() - start;

    assert.deepEqual(values, []);
    assert.ok(elapsed < 1000, `selector parsing took ${elapsed}ms`);
  }, 5000);

  it("parses CSS attribute selector flags without requiring whitespace", () => {
    assert.deepEqual(cssSelectorValues('[data-pw="Save"i] [data-pw="Cancel" s]', ["data-pw"]), [
      { attribute: "data-pw", operator: "=", value: "Save" },
      { attribute: "data-pw", operator: "=", value: "Cancel" },
    ]);
  });
});
