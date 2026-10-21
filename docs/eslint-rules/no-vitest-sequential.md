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

This rule has no options.

## Fix

Remove `.sequential` and isolate the fixture/state that requires ordering.

## Limits

A renamed import, a namespace member, a string or expression-free template key, optional chaining, a comma callee, `Function.prototype` forwarding, and one `const` alias are the same `.sequential` call. Vitest's default import is not a runner. A computed identifier key is not a static name. A module-scope binding named `test`, `it`, or `describe` is still a runner, including `const test = base.extend({})`, `import { baseTest as test } from "./fixtures"`, `import test from "./fx"`, `const { describe } = require("vitest")`, and a module-scope `function`, `let`, or `var`. A parameter or other nested-scope shadow, such as `function helper(suite)`, is not. `const test = createTest(); test("x", { sequential: true })` still uses the local name. `test.sequential.each(rows)(name, fn)` reports once. `const suite = v.describe; suite.sequential(...)` and `const t = test; t.sequential(...)` follow that one `const` alias.

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

## Related rules

- [`test-no-shared-state`](test-no-shared-state.md) identifies the mutable state
  that commonly motivates serial tests.
