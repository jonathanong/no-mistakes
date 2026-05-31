# `no-mistakes/playwright-assertion-timeout-cap`

Caps Playwright assertion timeouts.

Why: long assertion timeouts slow targeted test feedback and can hide flaky
waiting behavior.

Counterexample: `await expect(locator).toBeVisible({ timeout: 60000 })`.

Fix: remove custom timeouts or lower them to the configured maximum.
