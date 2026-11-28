# `no-mistakes/test-no-shared-state`

## Why

Mutable module-scope state leaks between tests and turns order or parallelism
into hidden input.

## Disallowed

```ts
let currentUser;
test("first", () => {
  currentUser = makeUser();
});
test("second", () => expect(currentUser).toBeDefined());
```

## Allowed

```ts
test("user", () => {
  const currentUser = makeUser();
  expect(currentUser).toBeDefined();
});
```

## Options

- `allowBeforeAllAssignments` defaults to `false`. Set it to `true` to count
  recognized `beforeAll` resets as cleanup for the owning suite when it runs
  in parallel. Serial scopes already count these resets. [Suite cleanup](https://github.com/jongleberry/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/packages/eslint-plugin-no-mistakes/src/rules/test-no-shared-state-cleanup.js#L12-L25) and [scope handling](https://github.com/jongleberry/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/packages/eslint-plugin-no-mistakes/src/rules/test-no-shared-state-cleanup.js#L49-L57).
- `unanalyzable` is `"finding"` or `"ignore"` and defaults to `"finding"`. A
  `.test`, `.it`, or `.describe` call whose receiver comes from a non-static
  `require` reports `unanalyzable`. `.skip` and `.each` on that call, such as
  `vt.test.skip(...)` and `vt.test.each(rows)(...)`, report `unanalyzable` too. A destructured `test`, `it`, or `describe`
  from that require, such as `const { test: check } = require(specifier)`, reports
  `unanalyzable`, including `check.skip(...)`. A `const` alias of that binding, such as
  `const run = check` and `const again = mid` after `const mid = check`, reports
  `unanalyzable` too. A `let` alias does not. `const check = require(specifier).test`
  followed by `check.skip(...)` does too, and so does `const run = check`. A computed modifier such as
  `check[modifier](...)` stays silent. `const { ["runner"]: runner } = require(moduleName); runner.test(...)` stays silent, while `const { runner } = require(moduleName)` does not. A later write of that destructure, such as
  `let { test: check } = require(specifier); check = localHelper`, does not.
  A destructured `expect` does not. A computed key such as
  `const { ["test"]: check } = require("vitest")` is not a test registration.
  `const check = require(spec).test` and `let vt = require(spec); vt.test(...)`
  report `unanalyzable`. A stable `let { runner } = require(specifier); runner.test(...)` does too. A later write of `runner`, and `let { [key]: named }`, do not.

`const vt = require("vitest")`, `const { test } = require("vitest")`,
`const check = require("vitest").test`,
`import vt = require("vitest")`, and `const { test } = require("@playwright/test")`
register tests. A later write, such as `let { test: check } = require("vitest"); check = local`,
does not enter test scope. Vitest's default import is not a test callee. A type assertion
on an array, such as `[] as string[]`, is still a mutable initializer, and
`list = [] as string[]` inside `beforeEach` is still a reset. `new (Map as typeof Map)()` is still a mutable Map. A type-only default import, such as `import type vt from "vitest"`, is not a namespace. A push through a
type assertion, a non-null assertion, or a cast method, such as
`(bag as any[]).push(1)`, `bag!.push(1)`, and `(bag.push as any)(1)`, is still
that shared mutation. A named callback wrapped in `as` or `!`, such as
`test("case", run as typeof run)` or `test("case", run!)`, is still that callback.
A trailing non-callback, such as `test("case", run, timeout)` or `test("case", run, timeout as number)`, does not replace it.
A suppression on `vt.test` silences the unresolved-registration advisory.
A nested pattern such as `const { pkg: { vt } } = require("vitest")` is not traced
to the inner binding.

## Fix

Create mutable values inside each test or reset them in the lifecycle hook that
owns the test isolation boundary.

For a forwarded registration, keep the registration and move the mutable value
into its callback. For example, rewrite
`test.apply.apply(test, [null, ["case", () => { shared.push(1); }]])` to
`test.apply.apply(test, [null, ["case", () => { const local = []; local.push(1); }]])`.
The same callback-local rewrite applies to direct calls, `.call`, and literal
`.apply` lists; suite and hook forwarding does not extend cleanup to sibling tests.

## Limits

A `mock*` binding read by a Vitest `vi.mock` factory stays quiet when a test assigns it. That includes `(vi as typeof vi).mock`, `vi["mock" as const]`, `const { vi: runner } = require("vitest"); runner.mock`, and `vi.mock("./dependency", (() => ({ mockItems })) as any)`. A namespace `v.test`, a Playwright default import, `test["only"]`, optional chaining, and one top-level `const` alias are the same test callback. A top-level `const suite = v.describe` is still that suite, so a hook inside it does not cover tests outside it. A comma callee such as `(0, v.describe)` is that same suite. `const suite = v.describe.serial` keeps the serial modifier. A nested binding that reuses a namespace import's local name is not that namespace, including a cast such as `(v as any).test`. Namespace setup hooks such as `v.beforeEach` are the same cleanup boundary as a named import. Vitest's default import is not a runner. A computed identifier key is not a static modifier. A computed identifier method, such as `entries[set]("x")`, is not a mutating call. A default import does not capture a factory reference: `import vt from "vitest"; const alias = vt; alias.mock("./dep", () => ({ mockItems }))` still reports a later `mockItems.push(1)`, including a nested `const vt = alias`. `const runner = require(specifier); const check = runner.test` and `const { test: check } = runner` report `unanalyzable` when `specifier` is not static. `let runner = require(specifier)` stays silent. A top-level `const check = v.test` is recorded, so a nested parameter named `check` is not that test. `v.describe.configure({ mode: "serial" })` marks the file serial. A nested function that another call invokes, such as `(consume as any)(update())`, is still checked.

Static string and expression-free template `call` / `apply` members register
that same test, suite, or hook. A literal `.apply` list may contain an inline or
named callback. For example, `test["call"](null, "case", callback)` and
`test.apply(null, ["case", callback])` share the ordinary test boundary.
`describe.apply(null, ["suite", callback])` owns only the hooks and tests inside
that callback; its cleanup does not cover sibling tests. Forwarded `beforeEach`
and `afterEach` callbacks keep their ordinary cleanup boundaries.
Nested intrinsic forwarding, such as
`test.apply.apply(test, [null, ["case", callback]])`, retains the final callback
boundary when each forwarded argument list is a literal array. Arbitrary nested
arrays do not register their contained functions.

An opaque `.apply` list, a sparse list, a list containing a spread, or a `.call`
argument spread is not followed. A function supplied only as the forwarding
receiver is not a registered callback. Parameter-default destructuring patterns are not
inferred: `function run({ test } = runner)` can receive another object.

A runner re-exported by another file is not visible here.

```ts
import { check } from "./barrel";
check("hidden", () => {
  counter += 1;
});
```

`barrel` may be `export { test as check } from "vitest"` or `export * from "vitest"`.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/test-no-shared-state -- suite intentionally tests module singleton initialization
sharedConnection = createConnection();
```

`no-mistakes-disable-line`, `no-mistakes-disable-next-line`, and
`no-mistakes-disable-file` also suppress this rule when the comment names
`test-no-shared-state` or `no-mistakes/test-no-shared-state`.
Directive-looking string literals do not.

## Related rules

- [`no-vitest-sequential`](no-vitest-sequential.md) prevents serial ordering
  from masking shared state.
