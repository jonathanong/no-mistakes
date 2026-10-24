# `no-mistakes/test-no-skips`

## Why

Skipped and exclusive tests can make a green test run hide missing coverage.
Conditional skips also let required credentials or setup failures silently
remove a test from CI. Lint only the test files you choose, so production code
and unrelated fixtures are unaffected.

## Disallowed

By default, the rule reports Vitest and Playwright test modifiers `skip`,
`skipIf`, `runIf`, `only`, `todo`, and `fixme`. It recognizes named, aliased,
and namespace imports from `vitest` and `@playwright/test`, statically named
`const` bindings from CommonJS `require()` and awaited dynamic `import()` calls,
including defaults and nested object patterns, plus the unbound `it`, `test`,
and `describe` globals in files selected by ESLint. Member chains
such as `test.skip.each(cases)(...)`, `test.concurrent.only(...)`, and
`test.only.for(cases)(...)` are checked, as are Playwright forms such as
`test.describe.fixme(...)`. In Vitest callbacks, a call to the actual callback
context's `skip()` is also reported, including a top-level destructured `skip`
binding (with a rename or default) and a stable local `const` captured or
destructured from the callback context. A `.each` callback receives row data,
while `.for` receives its test context as the second callback parameter; only
the latter is treated as a context `skip()`.

```ts
import { test } from "vitest";

test("hidden", ({ skip: omit }) => omit()); // reported
test("also hidden", (ctx) => { const omit = ctx.skip; omit(); }); // reported
test.each([{ skip() {} }])("row data", ({ skip }) => skip()); // allowed
```

For Playwright, the rule also reports statically visible `skip` and `fixme`
`annotation` values in test and suite declaration details. It follows same-file
`const` aliases for the test/describe callee, details object, annotation object,
and annotation array. This rule only treats the Playwright `skip` and `fixme`
annotation types as coverage-removing declarations; other annotation values
are outside this rule's check. A no-substitution template literal is read like
a string. A conditional type is reported when every branch is a literal and at
least one branch is a `skip` or `fixme` literal. The finding names each banned
type once, in source order. Allowing `skip` still reports
`cond ? "skip" : "fixme"`. A branch that is not a literal, including a dynamic
value, leaves that conditional unreported. A modifier that does not itself skip
the test, such as `only`, does not hide an annotation on that call. `skip` and
`fixme` modifiers already remove the test, so an annotation on that same call
is not reported again. This is syntax analysis, not runtime evaluation: mutable
bindings, computed or spread properties, and other dynamic annotation values
are not resolved and may not be reported. `test.describe.configure` and `tag`
do not remove the test, so they are ignored. It does not prove aliased objects
are immutable; mutation through another alias or a helper call is not tracked.
`test.step.skip()` skips only a step, so this rule leaves it alone.

```ts
import { test } from "@playwright/test";

test("known flake", { annotation: { type: "skip" } }, async ({ page }) => {});
test.describe("blocked suite", { annotation: [{ type: "fixme" }] }, () => {});
test.only("focused and skipped", { annotation: { type: `skip` } }, async () => {});

// `configure` and `tag` do not remove the test.
test.describe.configure({ mode: "serial" });
test("tagged", { tag: "@slow", annotation: { type: "issue" } }, async () => {});
```

Same-file `const test = <binding>.extend(...)` (and chained
`.extend().extend()`) remains the original Playwright or Vitest `test`
binding, including declaration annotations. Arguments passed to `extend` itself
are fixture options, not test details, so an annotation object there is not a
declaration skip. Cross-file fixture modules are not
inferred and have no hardcoded path conventions: list their import specifiers
in `testModules`. Only the exported `test` from those modules is admitted as
Playwright's `test`. For namespace bindings, only `fixtures.test` is admitted;
`fixtures.describe` and `fixtures.it` stay ordinary. This applies to ESM,
CommonJS, and awaited dynamic imports. A named `describe` import from the same
module also stays ordinary. A fixtures module re-exporting Playwright's
`describe` as a separate named member is not followed; `fixtures.test.describe`
is the supported form.

`const` destructuring follows a default and a nested object pattern when the
key that introduces the identifier is `it`, `test`, or `describe`:

```ts
const { test: t = fallback } = require("vitest");
const { runner: { test: nested } = fallback } = require("@playwright/test");

t.skip("defaulted rename", () => {});
nested.fixme("nested default", () => {});
```

A rest element, an array element, a `let` binding, and a computed key that is
not a string literal are not test bindings. A nested key such as `skip` is the
modifier function, not `test`.

```ts
import { test as check, describe } from "vitest";
import { test as pw } from "@playwright/test";

check.skipIf(!process.env.API_KEY)("requires API access", () => {});
describe.only("temporary focus", () => {});
pw.fixme("known issue", async ({ page }) => {});

const browser = pw.extend({ account: async ({}, use) => { await use("owner"); } });
browser.skip("same-file fixture skip", async () => {});
```

## Allowed

The rule follows call identity and syntax rather than property spelling. Data
fields such as `options.skip`, ordinary object methods, and names such as
`skipIfDeleted()` are allowed. A function's unrelated `skip()` method is also
allowed; Vitest's bound callback context `context.skip()` and its bound local
aliases are not. Mutable local captures remain unresolved.

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
modifier produces no finding in supported import, global, or member-chain
forms; allowing `skip` or `fixme` also suppresses the matching Playwright
declaration-annotation finding.

```js
"no-mistakes/test-no-skips": ["error", { allow: ["todo"] }]
```

`testModules` names import specifiers whose exported `test` is Playwright's,
matching the specifier as written. It defaults to `[]`. List every specifier
form the tests use: `./fixtures` and `../fixtures` are different entries, and
the option applies to the whole config scope rather than one directory.
A default import stays silent even when its specifier is listed, because only
the exported `test` binding is admitted.

```js
"no-mistakes/test-no-skips": ["error", { testModules: ["./fixtures", "../fixtures", "@app/test"] }]
```

```ts
import { test } from "./fixtures";
import { test as parent } from "../fixtures";
import defaultTest from "./fixtures";

test.skip("configured fixture skip", async () => {});
parent.skip("parent specifier is a separate entry", async () => {});
test("configured annotation skip", { annotation: { type: "skip" } }, async () => {});
defaultTest.skip("default import is not the exported test", async () => {});
```

A fixture module omitted from `testModules` stays silent:

```ts
import { test } from "./fixtures";
test.skip("not reported without testModules", async () => {});
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

Remove the modifier or the `skip` / `fixme` declaration annotation and make
the test runnable. If a test needs credentials, provide them in the test
environment and fail setup clearly when required credentials are missing.
Replace timer-based waits with an explicit completion signal or test fixture
barrier. For a known defect, keep an ordinary failing test that records the
behavior to restore; use `todo` only when the team has explicitly allowed it.

A same-file default import of `@playwright/test` is `test`. A string or expression-free template member, and `const { test } = namespace`, are the same binding. Vitest's default import is not a runner. Another file's re-export is not visible here: `import { check } from "./barrel.mjs"` stays unreported when `barrel.mjs` is `export { test as check } from "vitest"` or `export * from "vitest"`. A parameter default such as `function run({ test } = vitest)` is not judged, because the caller can pass a different object.

## Immutable computed keys

A computed key can follow a same-file `const` initialized by a string literal,
an expression-free template, or another such `const`. Resolution uses the actual
lexical binding, including shadowing, and applies through recognized runner aliases,
computed API chains and `test.extend(...)` results:

```ts
import { test } from "vitest";
const modifier = "skip";
const check = test.extend({});
check[modifier]("hidden", () => {}); // reported
```

Imported keys, parameters, `let`/`var`, destructured bindings, reassigned bindings, cycles,
and runtime expressions remain unresolved. For example, `test[key](...)` inside
`function run(key) { ... }` does not use an outer `const key = "skip"`.

## Limits

Another file's re-export is not visible here.

```ts
import { check } from "./barrel.mjs";
check.skip("hidden", () => {});
```

`barrel.mjs` may be `export { test as check } from "vitest"` or `export * from "@playwright/test"`. A parameter default such as `function run({ test } = vitest)` is not judged, because the caller can pass a different object.

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
