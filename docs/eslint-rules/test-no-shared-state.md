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

## Suppression

```ts
// eslint-disable-next-line no-mistakes/test-no-shared-state -- suite intentionally tests module singleton initialization
sharedConnection = createConnection();
```

## Related rules

- [`no-vitest-sequential`](no-vitest-sequential.md) prevents serial ordering
  from masking shared state.
