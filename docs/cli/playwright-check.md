# `no-mistakes playwright check`

Check Playwright route and selector coverage.

```sh
no-mistakes playwright check --root . --json
```

Use this when an agent changes pages, selectors, or tests and needs project-wide
coverage validation.

When an uncovered selector value is present only as a helper-wrapper argument,
the diagnostic points at that wrapper call, but wrapper calls do not satisfy
selector coverage. Inline a literal `getByTestId(...)` call or add explicit
wrapper support.

Shared Playwright options are documented in [`playwright`](playwright.md).

Node API: `playwrightCheck(options)`.
