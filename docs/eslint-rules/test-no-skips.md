# `no-mistakes/test-no-skips`

## Why

Skipped and exclusive tests can make a green test run hide missing coverage.
Conditional skips also let required credentials or setup failures silently
remove a test from CI. Lint only the test files you choose, so production code
and unrelated fixtures are unaffected.

## Disallowed

By default, the rule reports Vitest and Playwright test modifiers `skip`,
`skipIf`, `runIf`, `only`, `todo`, and `fixme`. It recognizes named, aliased,
and namespace imports from `vitest` and `@playwright/test`, plus the unbound
`it`, `test`, and `describe` globals in files selected by ESLint. Member chains
such as `test.skip.each(cases)(...)`, `test.concurrent.only(...)`, and
`test.only.for(cases)(...)` are checked, as are Playwright forms such as
`test.describe.fixme(...)`. In Vitest callbacks, a call to the actual callback
context's `skip()` is also reported. A `.each` callback receives row data,
while `.for` receives its test context as the second callback parameter; only
the latter is treated as a context `skip()`.

```ts
import { test as check, describe } from "vitest";
import { test as pw } from "@playwright/test";

check.skipIf(!process.env.API_KEY)("requires API access", () => {});
describe.only("temporary focus", () => {});
pw.fixme("known issue", async ({ page }) => {});
```

## Allowed

The rule follows call identity and syntax rather than property spelling. Data
fields such as `options.skip`, ordinary object methods, and names such as
`skipIfDeleted()` are allowed. A function's unrelated `skip()` method is also
allowed; Vitest's bound callback context `context.skip()` is not.

```ts
const options = { skip: "explain a skipped result" };
function skipIfDeleted() {}

test("loads the page", async ({ page }) => {
  await page.goto("/items");
});
```

## Options

`allow` is a list of modifiers to permit. It defaults to `[]`; entries must be
one of `skip`, `skipIf`, `runIf`, `only`, `todo`, or `fixme`. An allowed
modifier produces no finding in any supported import, global, or member-chain
form.

```js
"no-mistakes/test-no-skips": ["error", { allow: ["todo"] }]
```

## Scope

This is a single-file ESLint/Oxlint rule. It does not discover a repository's
tests or infer framework conventions from a project configuration. Use flat
config `files` globs to select the test files that should be checked:

```js
import noMistakes from "eslint-plugin-no-mistakes";

export default [{
  files: ["**/*.test.*", "tests/**"],
  plugins: { "no-mistakes": noMistakes },
  rules: { "no-mistakes/test-no-skips": "error" },
}];
```

Adjust the patterns to match the repository's test locations. Files outside
these globs are not checked by this rule.

## Fix

Remove the modifier and make the test runnable. If a test needs credentials,
provide them in the test environment and fail setup clearly when required
credentials are missing. Replace timer-based waits with an explicit completion
signal or test fixture barrier. For a known defect, keep an ordinary failing
test that records the behavior to restore; use `todo` only when the team has
explicitly allowed it.

## Suppression

Use a `no-mistakes` directive for a reviewed exception:

```ts
// no-mistakes-disable-next-line test-no-skips -- covers the documented disabled-account flow
test.skip("disabled account", () => {});
```

`no-mistakes-disable-line` suppresses a finding on that line, and
`no-mistakes-disable-file` opts out the whole file when it appears before the
first code token. Directives accept either `test-no-skips` or
`no-mistakes/test-no-skips` as the rule name and are specific to `no-mistakes`
rules. Prefer removing the modifier when the skip is no longer intentional.

## Related rules

- [`no-vitest-sequential`](no-vitest-sequential.md) keeps tests independent of
  execution order.
- [`test-no-shared-state`](test-no-shared-state.md) identifies shared mutable
  state that can motivate skipped or serialized tests.
