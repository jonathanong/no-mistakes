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

For `page.goto` and `page.waitForURL`, URL templates fold same-file `const`
bindings initialized with a string literal or an expression-free template:
`const slug = 'post'; await page.goto(`/blog/${slug}`)` covers `/blog/post`.
Every interpolation must resolve to such a binding. Parameters, imports,
`let`/`var`, non-string initializers, writes, and reads before initialization
retain the unresolved template, preserving dynamic-route matching.

Shared Playwright options are documented in [`playwright`](playwright.md).

Node API: `playwrightCheck(options)`.

Optional [integration route coverage sources](../configuration/integration-route-coverage.md)
retain the registered Vitest entry, declaration module, and framework provenance
in route coverage details. Configuration omitted preserves existing behavior.
