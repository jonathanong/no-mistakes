import { describe, expect, it } from "vitest";
import { fixture } from "./helpers.mjs";
import { pluginFindings, snapshotCases, snapshotDocument } from "./diagnostic-snapshot.mjs";

function recallCase(fixtureName, rule, filename = "fixture.jsx", options) {
  return {
    name: fixtureName,
    file: filename,
    rule,
    ...(options === undefined ? {} : { options }),
    findings: pluginFindings(fixture(fixtureName), rule, options, filename),
  };
}

const cases = [
  recallCase(
    "playwright-assertion-timeout-cap/rest-binding.spec.ts",
    "playwright-assertion-timeout-cap",
    "e2e/playwright-assertion-timeout-cap/rest-binding.spec.ts",
  ),
  recallCase(
    "playwright-assertion-timeout-cap/expect-cycle.spec.ts",
    "playwright-assertion-timeout-cap",
    "e2e/playwright-assertion-timeout-cap/expect-cycle.spec.ts",
  ),
  recallCase(
    "playwright-require-interactive-test-id/unresolved-alias.jsx",
    "playwright-require-interactive-test-id",
    "e2e/playwright-require-interactive-test-id/unresolved-alias.jsx",
    { interactiveComponents: ["Button"] },
  ),
  recallCase(
    "playwright-require-interactive-test-id/shadowed-kinds.jsx",
    "playwright-require-interactive-test-id",
    "e2e/playwright-require-interactive-test-id/shadowed-kinds.jsx",
    { interactiveComponents: ["Button"] },
  ),
  recallCase(
    "playwright-prefer-get-by-test-id/rest-binding.jsx",
    "playwright-prefer-get-by-test-id",
    "e2e/playwright-prefer-get-by-test-id/rest-binding.jsx",
  ),
  recallCase(
    "playwright-selector-priority/unknown-bindings.spec.js",
    "playwright-selector-priority",
    "e2e/playwright-selector-priority/unknown-bindings.spec.js",
  ),
  recallCase(
    "playwright-no-hoisted-unique-token/rest-factory.spec.js",
    "playwright-no-hoisted-unique-token",
    "e2e/playwright-no-hoisted-unique-token/rest-factory.spec.js",
    { tokenFactories: ["randomSuffix"] },
  ),
  recallCase(
    "playwright-no-hoisted-unique-token/loop-binding.spec.js",
    "playwright-no-hoisted-unique-token",
    "e2e/playwright-no-hoisted-unique-token/loop-binding.spec.js",
    { tokenFactories: ["randomSuffix"] },
  ),
  recallCase(
    "playwright-no-raw-scroll-pagination/loop-binding.spec.js",
    "playwright-no-raw-scroll-pagination",
    "e2e/playwright-no-raw-scroll-pagination/loop-binding.spec.js",
  ),
  recallCase(
    "playwright-no-raw-scroll-pagination/shadowed-cursor.spec.js",
    "playwright-no-raw-scroll-pagination",
    "e2e/shadowed-cursor.spec.js",
  ),
  recallCase("playwright-literals/recall.jsx", "playwright-literals"),
  recallCase("playwright-prefer-get-by-test-id/recall.jsx", "playwright-prefer-get-by-test-id"),
  recallCase(
    "playwright-prefer-get-by-test-id/suppressed-drag.jsx",
    "playwright-prefer-get-by-test-id",
  ),
  recallCase(
    "playwright-selector-priority/recall.spec.js",
    "playwright-selector-priority",
    "e2e/recall.spec.js",
  ),
  recallCase(
    "playwright-assertion-timeout-cap/recall.spec.ts",
    "playwright-assertion-timeout-cap",
    "e2e/recall.spec.ts",
  ),
  recallCase(
    "playwright-no-raw-scroll-pagination/recall.spec.js",
    "playwright-no-raw-scroll-pagination",
    "e2e/recall.spec.js",
  ),
  recallCase(
    "playwright-no-raw-scroll-pagination/forwarded-cursor.spec.js",
    "playwright-no-raw-scroll-pagination",
    "e2e/forwarded-cursor.spec.js",
  ),
  recallCase(
    "playwright-require-exported-component-attribute/recall.tsx",
    "playwright-require-exported-component-attribute",
    "recall.tsx",
  ),
  recallCase(
    "playwright-require-interactive-test-id/recall.jsx",
    "playwright-require-interactive-test-id",
    "fixture.jsx",
    { interactiveComponents: ["SaveButton"] },
  ),
  recallCase(
    "playwright-no-hoisted-unique-token/recall.spec.js",
    "playwright-no-hoisted-unique-token",
    "playwright/recall.spec.js",
    { tokenFactories: ["randomSuffix"] },
  ),
  recallCase("playwright-prefer-get-by-test-id/branches.jsx", "playwright-prefer-get-by-test-id"),
  recallCase(
    "playwright-no-hoisted-unique-token/branches.spec.js",
    "playwright-no-hoisted-unique-token",
    "playwright/branches.spec.js",
    { tokenFactories: ["randomSuffix"] },
  ),
  recallCase(
    "playwright-no-raw-scroll-pagination/branches.spec.js",
    "playwright-no-raw-scroll-pagination",
    "e2e/branches.spec.js",
  ),
  recallCase(
    "playwright-assertion-timeout-cap/branches.spec.ts",
    "playwright-assertion-timeout-cap",
    "e2e/branches.spec.ts",
  ),
  recallCase(
    "playwright-require-interactive-test-id/before-import.jsx",
    "playwright-require-interactive-test-id",
    "fixture.jsx",
    { interactiveComponents: ["SaveButton"] },
  ),
  recallCase(
    "playwright-require-interactive-test-id/branches.jsx",
    "playwright-require-interactive-test-id",
    "fixture.jsx",
    { interactiveComponents: ["Button", "Card"] },
  ),
  recallCase(
    "playwright-test-timeout-cap/binding-branches.ts",
    "playwright-test-timeout-cap",
    "e2e/binding-branches.ts",
  ),
  recallCase("playwright-literals/branches.jsx", "playwright-literals"),
  recallCase(
    "playwright-selector-priority/branches.spec.js",
    "playwright-selector-priority",
    "e2e/branches.spec.js",
  ),
  recallCase(
    "playwright-selector-priority/using-binding.spec.ts",
    "playwright-selector-priority",
    "e2e/using-binding.spec.ts",
  ),
  recallCase(
    "playwright-selector-priority/bare-locator.spec.js",
    "playwright-selector-priority",
    "e2e/bare-locator.spec.js",
  ),
  recallCase(
    "playwright-no-raw-scroll-pagination/window-member-alias.spec.js",
    "playwright-no-raw-scroll-pagination",
    "e2e/window-member-alias.spec.js",
  ),
  recallCase(
    "playwright-selector-priority/single-write-alias.spec.js",
    "playwright-selector-priority",
    "e2e/single-write-alias.spec.js",
  ),
  recallCase(
    "playwright-prefer-get-by-test-id/single-write-alias.jsx",
    "playwright-prefer-get-by-test-id",
  ),
  recallCase("playwright-literals/single-write-alias.jsx", "playwright-literals"),
  recallCase(
    "playwright-no-raw-scroll-pagination/single-write-alias.spec.js",
    "playwright-no-raw-scroll-pagination",
    "e2e/single-write-alias.spec.js",
  ),
  recallCase(
    "playwright-no-hoisted-unique-token/single-write-alias.spec.js",
    "playwright-no-hoisted-unique-token",
    "playwright/single-write-alias.spec.js",
    { tokenFactories: ["randomSuffix"] },
  ),
  recallCase(
    "playwright-assertion-timeout-cap/single-write-alias.spec.ts",
    "playwright-assertion-timeout-cap",
    "e2e/single-write-alias.spec.ts",
  ),
];

describe("playwright same-file recall", () => {
  it("snapshots diagnostics", async () => {
    await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
      "snapshots/recall-playwright.snap",
    );
  });
});
