---
name: testing-rules
description: "MUST NOT / SHOULD NOT rules for automated tests: assertions, mocks, fixtures, isolation, timing, snapshots, and coverage, for Vitest, Jest, backend integration, component, Swift, and .NET tests. Use when writing, reviewing, or fixing a test. Browser tests also load playwright-rules."
---

# Testing rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

## Assertions

- MUST NOT write an assertion that cannot fail (literal, self-comparison, mock echo, only checking a value is defined); assert an outcome a defect would change.
- MUST NOT run production code in a test without asserting its outcome.
- MUST NOT name a test for behavior it never asserts.
- MUST NOT match error message text; assert type, code, or structured fields.
- MUST NOT raise coverage with weak tests or exclude files from coverage; strengthen the test or refactor the code.
- MUST NOT treat a passing rerun as a fix for a flaky test; find the cause.
- MUST NOT assert something about every item in a collection without asserting it is non-empty; a per-item assertion passes over zero items. Asserting the collection equals `[]` is fine.
- MUST NOT accept "either outcome is fine" when one outcome is a degraded fallback.
- MUST NOT write a regression test whose fixture never reaches the bug's precondition; confirm it fails without the fix.
- SHOULD NOT add a test that locks in a branch you believe is unreachable; delete the branch or prove it reachable.
- SHOULD NOT trust a green exit code; check the number of tests that ran.
- SHOULD NOT keep redundant tests; keep one failure-sensitive test per contract at the lowest realistic boundary.
- SHOULD NOT recompute the production algorithm in the test; a shared bug cancels out.
- SHOULD NOT assert what a dependency owns (its messages, markup, call order, exact versions or pins); assert your own outcome.
- SHOULD NOT freeze documentation prose in tests unless a consumer parses that exact text.

## Mocks and boundaries

- MUST NOT mock the module under test; mock transports, clocks, providers, and environment at the boundary.
- MUST NOT replace a mocked module's whole export object; spread the real module and override what you need.
- MUST NOT leave spies, env, timers, or globals unrestored after a test.
- MUST NOT use call-order fakes for calls whose order you do not own; route fakes by method and path.
- MUST NOT add test-only branches or placeholders to production code.
- MUST NOT patch a name the code under test never looks up (the definition instead of the import site); prove the patch is live by asserting a call on a path that should make one, even when the test's contract is no calls.
- MUST NOT patch a process-wide global and assert an exact call count; other code in the process calls it too.
- MUST NOT test that an import has no side effect from a process that already imported it.
- SHOULD NOT mock in integration tests, or rely on mock-only tests where wiring, serialization, transactions, or retries are the contract.
- SHOULD NOT keep a production export only so tests can reach it; test through the public surface.

## Isolation and timing

- MUST NOT leave `.only`, `.skip`, `.todo`, or conditional skips; fix or delete the test.
- MUST NOT share mutable module-scope state between tests, or depend on test order; build state per test.
- MUST NOT mark suites sequential to hide shared state; remove the shared state.
- MUST NOT sleep or use timing waits; await the specific condition with a named deadline.
- MUST NOT read the real clock; inject one.
- MUST NOT let gating tests call live third parties; replay recorded responses.
- MUST NOT skip a test when a credential or service is missing in CI; fail it.
- MUST NOT assert "nothing happened" before the event that would have caused it has settled.
- MUST NOT leak workers, subscriptions, handles, or console output past the test.
- MUST NOT attach a rejection, event, or response observer after the action that triggers it; register first.
- MUST NOT raise or remove a timeout to hide slowness; fix the wait condition.
- MUST NOT use fixed identities in a shared store, or reset shared state broadly; randomize ids and clean only what the test owns.
- MUST NOT bind port 0 or real network services in unit tests.
- MUST NOT scan the live repository checkout from unit tests; use saved fixtures.
- MUST NOT change shared database schema or server settings inside tests; migrate before the suite.
- MUST NOT hardcode dates a test depends on being in the future or past; derive them from the injected clock.

## Fixtures and snapshots

- MUST NOT write fixture files inline (`writeFile`, `mkdir`) in test code; save them under a fixtures directory and copy them in.
- MUST NOT fix a bug without a regression test.
- MUST NOT accept a changed snapshot without reviewing the diff, or regenerate snapshots by default in CI.
- MUST NOT hardcode git SHAs, real email domains, or real credentials in fixtures; use reserved test values.
- SHOULD NOT build fixtures by bypassing public constructors; keep them minimal and typed.
- SHOULD NOT hide expectations inside shared helpers; helpers remove setup only.
- SHOULD NOT add aggregate test files that only import other tests.
- SHOULD NOT build a "nothing was forgotten" guard as an allow-list; use a deny-list so new files are covered by default.

## Platform notes

- MUST NOT mock navigation, headers, cookies, or server-only APIs inside the component under test; mock at the framework edge or move the test to the browser.
- MUST NOT let Swift test doubles deliver after `stopLoading()` or share state without synchronization.
- SHOULD NOT share process or filesystem state across .NET tests, or assert private implementation instead of the serialized contract.
- SHOULD NOT use a story as end-to-end setup; keep story data local and deterministic.
