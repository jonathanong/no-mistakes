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

## HTML IDs

HTML `id` values are coverage candidates only when
[`tests.playwright.selectors.htmlIds`](../configuration/tests.md) is enabled or
`id` is explicitly configured as a test ID or component selector attribute.
Enabling `playwright-unique-html-ids` does not add IDs to coverage; that rule
scans IDs independently so it can detect duplicates without widening
`playwright-coverage`.

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

Selector coverage does not infer wrapper bodies. Configure an argument-bearing
wrapper explicitly when a shared helper represents `getByTestId(...)`:

```yaml
tests:
  playwright:
    selectors:
      wrappers:
        - module: "@app/playwright-locators"
          export: getAsideLocator
          testIdArgument: 1
```

After configuration, a static ESM import followed by
`getAsideLocator(page, 'save')` covers the same selector as
`page.getByTestId('save')`. Import aliases, default imports, and namespace
imports are supported. Module identity follows the request's normal
JavaScript/TypeScript resolution for relative and NodeNext paths, tsconfig
aliases and `baseUrl`, package `imports`, and workspace exports. Bare packages
and package subpaths do not depend on npm, pnpm, Yarn, or Bun, and
`node_modules` is not scanned.

Resolver-equivalent declarations with different `testIdArgument` values are
ambiguous and do not create coverage.

Helpers without a declaration, shadowed bindings, dynamic arguments, CommonJS
calls, and invalid wrapper declarations do not create selector coverage. An
uncovered selector value found in an undeclared helper call still includes a
hint at that call. Either configure the wrapper or add a literal
`getByTestId(...)` assertion.
