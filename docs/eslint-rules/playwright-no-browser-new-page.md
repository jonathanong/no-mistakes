# `no-mistakes/playwright-no-browser-new-page`

## Why

Calling `browser.newPage()` inside a Playwright spec opens a new, unmanaged
browser context and page that bypass the suite's shared page fixture. Auth
state, base URLs, console and page-error listeners, tracing hooks, custom
routes, and automatic teardown registered by the shared fixture are skipped,
and unclosed pages can leak across workers.

Specs should receive the configured shared page fixture (for example `page` or
a project-specific fixture such as `appPage`) instead of calling
`browser.newPage()`. When a spec legitimately needs a second isolated session,
create an explicit context with `browser.newContext()` and close it in the test
or a fixture.

## Disallowed

```ts
// e2e/orders.spec.ts
import { test } from "@playwright/test";

test("loads orders", async ({ browser }) => {
  const page = await browser.newPage();
  await page.goto("/orders");
});
```

```ts
// e2e/orders-alias.spec.ts
import { test } from "@playwright/test";

test("opens page through alias", async ({ browser: localBrowser }) => {
  const { newPage } = localBrowser;
  await newPage();
});
```

```ts
// e2e/orders-dynamic.spec.ts
import { test } from "@playwright/test";

declare const method: string;

test("dynamic browser access is unanalyzable", async ({ browser }) => {
  await browser[method]();
});
```

## Allowed

```ts
// e2e/orders.spec.ts
import { test } from "./fixtures";

test("loads orders", async ({ appPage }) => {
  await appPage.goto("/orders");
});
```

```ts
// e2e/isolated-context.spec.ts
import { test } from "@playwright/test";

test("opens an isolated browser context", async ({ browser }) => {
  const context = await browser.newContext();
  const page = await context.newPage();
  await page.goto("https://example.test/public");
  await context.close();
});
```

```ts
// e2e/fixture-definition.ts
import { test as base } from "@playwright/test";

export const test = base.extend({
  appPage: async ({ browser }, use) => {
    const page = await browser.newPage();
    try {
      await use(page);
    } finally {
      await page.close();
    }
  },
});
```

## Options

The rule is opt-in and is not included in `configs.recommended` or
`configs.strict`.

```js
{
  files: ["e2e/**/*.spec.ts"],
  rules: {
    "no-mistakes/playwright-no-browser-new-page": [
      "error",
      {
        fixtureName: "appPage",
        registrationPackages: ["./fixtures"],
      },
    ],
  },
}
```

- `fixtureName` names the shared page fixture recommended in diagnostics. It
  defaults to `"page"` and must be a non-empty string.
- `browserFixtureName` names the Playwright browser fixture property inspected
  on admitted test and hook callbacks. It defaults to `"browser"` and must be a
  non-empty string.
- `registrationPackages` lists additional module specifiers (beyond
  `"@playwright/test"`) whose `test` and `mergeTests` exports are admitted
  Playwright registrars. It defaults to `[]`.
- `unanalyzable` controls undecidable shapes involving Playwright fixtures, the
  browser binding, or test callbacks. It defaults to `"finding"` so uncertain
  inputs are surfaced rather than silently passing; set `"ignore"` to report
  only definite `browser.newPage()` references.
- `message` replaces the diagnostic text for definite `browser.newPage()`
  findings.

## Fix

Receive the configured shared page fixture in the test callback instead of
calling `browser.newPage()`:

```ts
import { test } from "./fixtures";

test("loads orders", async ({ appPage }) => {
  await appPage.goto("/orders");
});
```

If the test intentionally needs a separate browser context, call
`browser.newContext()` and close it when done.

## Limits

This rule is file-local and analyzes admitted Playwright test and hook
registrations (`test`, `only`, `skip`, `fixme`, `fail`, `beforeEach`,
`beforeAll`, `afterEach`, `afterAll`, `test.extend(...)`, and `mergeTests(...)`)
from `"@playwright/test"` and configured `registrationPackages` (ESM imports
and CommonJS `require`). Fixture factory definitions inside `test.extend({ ... })`
are intentionally out of scope so projects can define their shared page fixture.

When `unanalyzable` is `"finding"` (the default), dynamic member access
(`browser[method]()`), dynamic or rest destructuring, reassigned bindings,
escaping `browser` or raw `fixtures` references, spread registration arguments,
partially bound callbacks, unresolved external callbacks, and dynamic registrar
properties are reported instead of silently passing.

## Suppression

```ts
import { test } from "@playwright/test";

test("dedicated browser.newPage contract", async ({ browser }) => {
  // no-mistakes-disable-next-line playwright-no-browser-new-page -- verifies raw browser.newPage behavior
  const page = await browser.newPage();
  await page.close();
});
```

`no-mistakes-disable-line` and `no-mistakes-disable-file` suppress findings the
same way. Qualified `no-mistakes/playwright-no-browser-new-page` directives and
standard ESLint suppression comments are also accepted.

## Related rules

- [`playwright-no-hoisted-unique-token`](playwright-no-hoisted-unique-token.md)
  keeps per-worker fixture tokens inside hooks.
- [`playwright-test-timeout-cap`](playwright-test-timeout-cap.md) caps or bans
  per-test deadline overrides.
- [`forbidden-member-chains`](forbidden-member-chains.md) bans static member
  chains lexically without Playwright fixture provenance.
