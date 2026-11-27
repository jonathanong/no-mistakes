# `no-mistakes/no-vitest-sequential`

## Why

Vitest sequential modifiers make tests depend on execution order and can mask
shared state that fails under normal parallel execution.

## Disallowed

```ts
// users.test.ts
describe.sequential("users", () => {
  test("creates", () => {});
});
```

## Allowed

```ts
// users.test.ts
describe("users", () => {
  test("creates", () => {});
});
```

## Options

`unanalyzable` is `"finding"` or `"ignore"` and defaults to `"finding"`. A
`require` or `import = require` of `vitest` whose specifier is not a static
string reports `unanalyzable` when it is used as a sequential registration,
instead of passing silently. A destructured `test`, `it`, or `describe` from
that require follows the same option instead of the built-in runner name. A
`const` alias of its `.sequential` member, such as `const serial = base.sequential`,
reports `unanalyzable` as well. A non-sequential alias stays quiet. A
computed member of a resolved Vitest module, such as `vt[name].sequential`,
stays silent. A nested `require("vitest")` does not make a different binding
of the same name a namespace.

A static `require("vitest")`, `import vt = require("vitest")`, and a
destructured `test`, `it`, or `describe` from that module are the same runner
as a named import. Vitest's default import is not a runner, and a destructured
`expect` is not a runner. Type assertions, `satisfies`, and `as const` keys do
not hide a sequential call.

## Fix

Remove `.sequential` and isolate the fixture/state that requires ordering.

TypeScript assertions, `satisfies`, non-null assertions, and generic
instantiations do not change whether a call runs sequentially. The rule also
checks through these wrappers on the runner, a same-file `const` alias, static
member/option keys, the options object, and the `true` option value:

```ts
// Disallowed: type-only wrappers do not make execution parallel.
(test.sequential as typeof test)("users", () => {});
test["sequential" as const]("users", () => {});
test("users", { sequential: true as const } satisfies object, () => {});

// Rewrite each call to run independently, with isolated fixture/state.
test("users", () => {});
```

For a wrapped alias, replace `const serial = test.sequential as typeof test`
with `const serial = test` (or call `test` directly). For wrapped option keys or
values, remove the `sequential` option. Type assertions around dynamic keys or
boolean variables do not make their runtime values statically known.

## Limits

A literal-array `.apply` forwards sequential options just like `.call`, including
`test["apply"](null, ["name", { sequential: true }, callback])` and a `const`
runner alias. An opaque argument array does not expose its options. A visible
`{ sequential: true }` argument still reports when a trailing `.call` spread
prevents full argument normalization. Rewrite
sequential registrations as parallel tests rather than hiding their options in a wrapper.

A renamed import, a namespace member, a string or expression-free template key, optional chaining, a comma callee, `Function.prototype` forwarding, and one `const` alias are the same `.sequential` call. Vitest's default import is not a runner. A type-only import, including `import type { test }` and `import { type it }`, is erased and is not a runner. A computed identifier key is not a static name. A module-scope binding named `test`, `it`, or `describe` is still a runner, including `const test = base.extend({})`, `import { baseTest as test } from "./fixtures"`, `import test from "./fx"`, `const { describe } = require("vitest")`, and a module-scope `function`, `let`, or `var`. A parameter or other nested-scope shadow, such as `function helper(suite)`, is not. `const test = createTest(); test("x", { sequential: true })` still uses the local name. `test.sequential.each(rows)(name, fn)` reports once. `const suite = v.describe; suite.sequential(...)` and `const t = test; t.sequential(...)` follow that one `const` alias.

Another file's re-export is not visible here.

```ts
import { suite } from "./barrel.mjs";
suite.sequential("hidden", () => {});
```

`barrel.mjs` may be `export { describe as suite } from "vitest"` or `export * from "vitest"`. A parameter default such as `function run({ describe } = vitest)` is not judged, because the caller can pass a different object.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/no-vitest-sequential -- verifies a legacy serialized migration harness
describe.sequential("migration", () => {});
```

`no-mistakes-disable-line`, `no-mistakes-disable-next-line`, and
`no-mistakes-disable-file` also suppress this rule when the comment names
`no-vitest-sequential` or `no-mistakes/no-vitest-sequential`.
Directive-looking string literals do not.

## Related rules

- [`test-no-shared-state`](test-no-shared-state.md) identifies the mutable state
  that commonly motivates serial tests.
