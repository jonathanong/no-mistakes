# `no-mistakes/test-no-shared-state`

## Why

Mutable module-scope state leaks between tests and turns order or parallelism
into hidden input.

## Disallowed

```ts
let currentUser;
test("first", () => { currentUser = makeUser(); });
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
  `require` reports `unanalyzable`.

`const vt = require("vitest")`, `const { test } = require("vitest")`,
`import vt = require("vitest")`, and `const { test } = require("@playwright/test")`
register tests. Vitest's default import is not a test callee. A type assertion
on an array, such as `[] as string[]`, is still a mutable initializer, and
`list = [] as string[]` inside `beforeEach` is still a reset.

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

A namespace `v.test`, a Playwright default import, `test["only"]`, optional chaining, and one top-level `const` alias are the same test callback. A top-level `const suite = v.describe` is still that suite, so a hook inside it does not cover tests outside it. A comma callee such as `(0, v.describe)` is that same suite. `const suite = v.describe.serial` keeps the serial modifier. A nested binding that reuses a namespace import's local name is not that namespace. Namespace setup hooks such as `v.beforeEach` are the same cleanup boundary as a named import. Vitest's default import is not a runner. A computed identifier key is not a static modifier. A top-level `const check = v.test` is recorded, so a nested parameter named `check` is not that test. `v.describe.configure({ mode: "serial" })` marks the file serial.

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
