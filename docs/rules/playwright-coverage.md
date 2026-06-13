# `playwright-coverage`

Runs Playwright route and selector coverage from `no-mistakes check`.

```yaml
rules:
  - rule: playwright-coverage
    tests:
      playwright: [web]
```

Counterexample: a page or selector is reachable in the app but has no matching
Playwright coverage.

Fix: add coverage, adjust selector config, or exclude intentional gaps.

## `getByTestId` and `testIdAttribute`

Coverage matching is attribute-aware: a `page.getByTestId('save')` assertion only
covers an app selector whose attribute matches the test's effective
`testIdAttribute`. `no-mistakes` resolves that attribute from the Playwright
config's `use.testIdAttribute`.

When the Playwright config is built by a helper function (e.g.
`defineConfig(createPlaywrightConfig({ ... }))`), `testIdAttribute` is set inside
the helper body and cannot be read statically. `no-mistakes` then falls back to
the configured [`tests.playwright.selectors.testIds`](../configuration/tests.md),
so `getByTestId('save')` still covers `data-pw="save"`. You can also declare the
attribute explicitly with
[`tests.playwright.testIdAttribute`](../configuration/tests.md#testidattribute),
which takes precedence over both.

## Helper wrappers

Selector coverage does not infer wrapper semantics. If a spec calls a helper such
as `getAsideLocator(page, 'save')` and that helper internally calls
`page.getByTestId(dataPw)`, the selector remains uncovered because the literal
`getByTestId('save')` call is not present in the spec.

When an uncovered selector value appears in a helper-wrapper call, the failure
includes a hint pointing at that call. Fix it by inlining the literal
`getByTestId(...)` assertion or by adding explicit wrapper support intentionally.
