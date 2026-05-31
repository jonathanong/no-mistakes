# `no-mistakes/playwright-no-set-timeout`

Disallows fixed sleeps in Playwright tests.

Why: fixed sleeps make tests slower and less deterministic than waiting for a
specific UI condition.

Counterexample: `await page.waitForTimeout(1000)`.

Fix: wait on locators, assertions, network state, or app-visible conditions.
