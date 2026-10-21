# `no-mistakes/no-vitest-sequential`

## Why

Vitest sequential modifiers make tests depend on execution order and can mask
shared state that fails under normal parallel execution.

## Disallowed

```ts
describe.sequential("users", () => {
  test("creates", () => {});
});
```

## Allowed

```ts
describe("users", () => {
  test("creates", () => {});
});
```

## Options

This rule has no options.

## Fix

Remove `.sequential` and isolate the fixture/state that requires ordering.

## Limits

A renamed import, a namespace member, a string or expression-free template key, optional chaining, a comma callee, `Function.prototype` forwarding, and one `const` alias are the same `.sequential` call. Vitest's default import is not a runner. A computed identifier key is not a static name. A nested binding that reuses the imported name, such as `function helper(suite)`, is not the imported runner.

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
