# `no-mistakes playwright check`

Check Playwright route and selector coverage.

```sh
no-mistakes playwright check --root . --json
```

Use this when an agent changes pages, selectors, or tests and needs project-wide
coverage validation.

Configured selector wrappers satisfy coverage when their declared argument is
a supported test-ID literal. Wrapper declarations live under
`tests.playwright.selectors.wrappers` and identify the imported module, export,
and zero-based argument index. Helper calls without a declaration remain hints
and do not create coverage.

Shared Playwright options are documented in [`playwright`](playwright.md).

Node API: `playwrightCheck(options)`.
