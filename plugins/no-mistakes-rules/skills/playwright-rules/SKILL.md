---
name: playwright-rules
description: "Load before writing, reviewing, or de-flaking a Playwright or other browser end-to-end test, or adding data-testid attributes to UI. Lists the defects to avoid and the patterns that need a stated reason: waits, locators, test ids, setup, network mocking, timeouts."
---

# Playwright rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

- MUST NOT use `waitForTimeout` or other sleeps; wait for a URL, response, DOM state, or visible signal.
- MUST NOT start a response or event wait after the action that triggers it; create the wait first.
- MUST NOT depend on test order or state left by another test; create fresh entities for anything a test mutates.
- SHOULD NOT run browser tests against a dev server; test a production build.
- MUST NOT assume `scrollIntoViewIfNeeded` triggers scroll-driven behavior; scroll for real.
- MUST NOT swallow failures from a locator that the test requires for an action or assertion.
- MUST NOT call `browser.newPage()` outside a fixture definition when a shared page fixture is available; receive the shared page fixture instead.
- MUST NOT hide flakiness with large assertion or test timeouts; fix the readiness condition.
- MUST NOT compute unique tokens at module or `describe` scope for use in hooks; generate them inside the hook.
- MUST NOT drive infinite scroll with one raw scroll call; scroll until the awaited request fires.
- MUST NOT duplicate HTML ids within the same document tree.
- SHOULD NOT reuse test ids without scoping locators that need to target a single element.
- SHOULD NOT log in or build state through the UI; use fixtures or APIs.
- SHOULD NOT mock the network except for third parties, fault injection, or states you cannot seed; seed data instead.
