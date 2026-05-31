# `no-mistakes playwright tests`

Print route, selector, and fetch assertions grouped by Playwright test.

```sh
no-mistakes playwright tests tests/e2e/users.spec.ts --json
```

Use this to inspect what a test proves before editing coverage expectations.

Shared Playwright options are documented in [`playwright`](playwright.md).

Node API: `playwrightTests(options)`.
