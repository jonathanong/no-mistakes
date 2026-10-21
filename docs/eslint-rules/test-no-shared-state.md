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

- `allowBeforeAllAssignments` permits module-scope assignments in `beforeAll`.
  It defaults to `false`.

## Fix

Create mutable values inside each test or reset them in the lifecycle hook that
owns the test isolation boundary.

## Limits

A namespace `v.test`, a Playwright default import, `test["only"]`, optional chaining, and one top-level `const` alias are the same test callback. A top-level `const suite = v.describe` is still that suite, so a hook inside it does not cover tests outside it. Namespace setup hooks such as `v.beforeEach` are the same cleanup boundary as a named import. Vitest's default import is not a runner. A computed identifier key is not a static modifier.

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

## Related rules

- [`no-vitest-sequential`](no-vitest-sequential.md) prevents serial ordering
  from masking shared state.
