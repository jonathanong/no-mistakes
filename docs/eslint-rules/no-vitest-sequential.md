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

## Suppression

```ts
// eslint-disable-next-line no-mistakes/no-vitest-sequential -- verifies a legacy serialized migration harness
describe.sequential("migration", () => {});
```

## Related rules

- [`test-no-shared-state`](test-no-shared-state.md) identifies the mutable state
  that commonly motivates serial tests.
