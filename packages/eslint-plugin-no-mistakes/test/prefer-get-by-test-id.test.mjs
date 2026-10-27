import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";

const ruleName = "playwright-prefer-get-by-test-id";
const languageOptions = { ecmaVersion: 2024, sourceType: "module" };

const suite = {
  valid: [
    { code: 'page.getByTestId("save")' },
    { code: 'page[method]("[data-pw=\\"computed\\"]")' },
    { code: "page.locator()" },
    { code: "page.locator(selector)" },
    { code: 'page.locator("")' },
    { code: 'page.click("[data-testid^=\\"user-\\"]")' },
    { code: 'const selector = "[data-pw=\\"ignored\\"]"' },
    { code: "page.locator(`${dynamic}`)" },
    { code: "page.locator(123)" },
    { code: 'page.locator(`[data-testid="${dynamic}"]`)' },
    { code: "page.locator(null)" },
    { code: 'page.locator(selector, "[data-pw=\\"ignored\\"]")' },
    { code: "page.dragAndDrop(selector, selector)" },
    { code: "page.dragAndDrop(123, ``)" },
    { code: 'page.locator("[data-pw]")' },
    {
      code: 'page.locator("[data-pw=\\"save\\"]", { hasText: "foo" })',
      options: [{ selectorAttributes: ["data-qa"] }],
    },
    { code: "page.locator(a, b)" },
    { code: "page.locator(``)" },
  ],
  invalid: [
    // One prefer finding on getByTestId("target").
    // Suppressing the source must leave the target selector visible.
    {
      code: fixture("playwright-prefer-get-by-test-id/suppressed-drag.jsx"),
    },
    {
      code: 'locator("[data-pw=\\"direct\\"]")',
    },
    {
      code: 'page.locator("[data-pw=\\"save\\"]")',
    },
    {
      code: "page.locator(\"[data-pw='open']\")",
    },
    {
      code: 'page.locator("[ data-pw = unquoted ]")',
    },
    {
      code: 'page.locator("[data-pw=flagged i]")',
    },
    {
      code: 'page.dragAndDrop("[data-testid=\\"source\\"]", "[data-pw=\\"target\\"]")',
    },
    {
      code: 'page.locator("[data-pw=\\"save\\"]", { hasText: "foo" })',
    },
    {
      code: 'page.locator("invalid", { has: page.locator("[data-pw=\\"ignored\\"]") })',
    },
    {
      code: 'page.dragAndDrop(selector, "[data-pw=\\"target\\"]")',
    },
    {
      code: 'page.dragAndDrop("[data-pw=\\"source\\"]", selector)',
    },
    {
      code: 'page.locator("[data-qa=\\"save\\"]")',
      options: [{ selectorAttributes: ["data-qa"] }],
    },
  ],
};

it("snapshots playwright-prefer-get-by-test-id diagnostics", async () => {
  await expect(
    snapshotDocument({
      ruleTester: snapshotSuite(ruleName, languageOptions, suite),
    }),
  ).toMatchFileSnapshot("snapshots/prefer-get-by-test-id.snap");
});
