# `no-mistakes/playwright-prefer-get-by-test-id`

Prefers `getByTestId` over CSS test-id selectors.

Why: semantic Playwright locators are easier to analyze and more resilient than
raw CSS selector strings.

Counterexample: `page.locator("[data-testid=save]")`.

Fix: replace CSS test-id selectors with `page.getByTestId()` or
`locator.getByTestId()`.
