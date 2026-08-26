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

## Multiple frontend apps

With more than one `type: nextjs` project configured, each rule application
must be bound to the app it covers — either via `projects:`:

```yaml
rules:
  - rule: playwright-coverage
    projects: [web]
    tests:
      playwright: [web]
```

or via [`tests.playwright.apps.<project>.project`](../configuration/tests.md#multiple-frontend-apps).
When `tests.playwright.apps` binds every Playwright project, a single unbound
rule application fans out over those apps. An unbound rule with more than one
candidate app and no `apps` map fails with an error naming the candidates.

Set `tests.playwright.coverage.routes: false` to disable uncovered-route
findings without dropping route selection edges. Selector findings stay on
unless `coverage.selectors` is also false.

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

## Why and when

Use this rule on a Next.js or Playwright surface when a passing test suite is
not enough evidence that every route and stable selector is exercised. It is
especially useful before shipping a new page, selector, or second frontend
app.

## What it catches/requires

The configured Playwright project must cover every selected route and selector
candidate. Route and selector coverage are independent, and the rule only uses
static relationships it can resolve from the configured app and test roots.

## Options and defaults

The rule has no rule-local `options` block. `tests.playwright.coverage.routes`
and `.selectors` default to `true`; set either to `false` to disable that class
of finding. `tests.playwright.selectors.htmlIds` defaults to `false`, and
`tests.playwright.testIdAttribute` overrides static config discovery when set.

## Valid example

```ts
test("saves a profile", async ({ page }) => {
  await page.goto("/profile");
  await page.getByTestId("save-profile").click();
});
```

## Counterexample

```tsx
export function ProfileForm() {
  return <button data-pw="save-profile">Save</button>;
}
```

If no configured Playwright test reaches this component or its route, the
selector remains uncovered.

## Fix

Add a static route/navigation or selector assertion, configure the correct
frontend app and selector roots, or disable only the intentionally untested
coverage class.

## Suppression

Use a top-of-file `no-mistakes-disable-file playwright-coverage` directive for
an intentional whole-file exception. Prefer `coverage.routes: false`,
`coverage.selectors: false`, or a narrower app/selector scope when the policy
does not apply to the entire rule.

## Related rules

See [`playwright-prefer-test-id-locators`](playwright-prefer-test-id-locators.md)
for locator quality and [`playwright-unique-test-ids`](playwright-unique-test-ids.md)
for selector identity. [`playwright-unique-html-ids`](playwright-unique-html-ids.md)
checks HTML IDs independently.
