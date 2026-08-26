# `no-mistakes/no-import-only-test-files`

## Why

An aggregate test file that only imports other tests hides test ownership and
duplicates the runner's built-in test discovery.

## Disallowed

```ts
import "./users.test";
import "./orders.test";
```

## Allowed

```ts
test("users are listed", () => {
  expect(listUsers()).toEqual([]);
});
```

## Options

This rule has no options.

## Fix

Let the runner discover each test file directly, or put real assertions in the
file rather than using it as an import-only manifest.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/no-import-only-test-files -- explicit integration test manifest required by this runner
import "./users.test";
```

## Related rules

- [`vitest-mock-test-file-naming`](vitest-mock-test-file-naming.md) keeps mock
  test ownership visible in filenames.
