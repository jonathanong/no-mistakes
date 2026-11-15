---
name: playwright-rules
description: "MUST NOT / SHOULD NOT rules for Playwright and other browser end-to-end tests: waits, locators, test ids, setup, network mocking, and timeouts. Use when writing, reviewing, or de-flaking a browser test or adding test ids to UI."
---

# Playwright rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

- MUST NOT use `waitForTimeout` or other sleeps; wait for a URL, response, DOM state, or visible signal.
- MUST NOT start a response or event wait after the action that triggers it; create the wait first.
- MUST NOT depend on test order or state left by another test; create fresh entities for anything a test mutates.
- SHOULD NOT run browser tests against a dev server; test a production build.
- MUST NOT assume `scrollIntoViewIfNeeded` triggers scroll-driven behavior; scroll for real.
- MUST NOT swallow locator failures (`isVisible().catch(() => false)`, `try/catch`); let a broken locator fail.
- MUST NOT hide flakiness with large assertion or test timeouts; fix the readiness condition.
- MUST NOT compute unique tokens at module or `describe` scope for use in hooks; generate them inside the hook.
- MUST NOT drive infinite scroll with one raw scroll call; scroll until the awaited request fires.
- MUST NOT duplicate HTML ids or test ids.
- SHOULD NOT log in or build state through the UI; use fixtures or APIs.
- SHOULD NOT mock the network except for third parties, fault injection, or states you cannot seed; seed data instead.
