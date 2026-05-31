# `no-mistakes/playwright-selector-priority`

Prefers semantic Playwright locators over raw selectors.

Why: semantic locators preserve user intent and reduce brittle CSS/XPath tests.

Counterexample: `page.locator(".btn.primary").click()`.

Fix: use role, label, text, placeholder, or test-id locators before raw CSS.
