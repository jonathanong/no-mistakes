# `no-mistakes/no-import-only-test-files`

## Why

An aggregate test file that only imports other tests hides test ownership and
duplicates the runner's built-in test discovery.

## Disallowed

```ts
// suite.test.ts
import "./users.test";
import "./orders.test";
```

## Allowed

```ts
// users.test.ts
test("users are listed", () => {
  expect(listUsers()).toEqual([]);
});
```

## Options

This rule has no options.

## Fix

Let the runner discover each test file directly, or put real assertions in the
file rather than using it as an import-only manifest.

## Limits

Immutable same-file `require` aliases, including `const load = require`, are
loader declarations rather than test work. A file containing only those aliases
and imports of test files is still an aggregate. `import name = require("./a.test")`,
`export * from "./a.test"`, `export { name } from "./a.test"`, and `require("./a.test")`
count, including a cast around `require`, the specifier, or the call result.
Shadowed or mutable loaders, computed identifier members, and opaque `.apply`
argument lists remain unknown.
Remove the aggregate and let the runner discover the test files directly.

An optional `require?.()` call and a static `require["call"]` or `require["apply"]` key name the same test file as `require.call`. A template with an expression, or a computed identifier key, does not.

A test file reached through another module's re-export is not visible here.

```js
require("./barrel");
```

`barrel.js` may be `module.exports = require("./users.test")`.

## Suppression

The rule reports one finding on the first statement. Place a line or next-line
directive on that statement. Each example below is a separate file.

```ts
// no-mistakes-disable-file no-import-only-test-files -- explicit integration test manifest required by this runner
import "./suite.test";
```

```ts
// no-mistakes-disable-next-line no-import-only-test-files -- explicit integration test manifest required by this runner
import "./users.test";
```

```ts
import "./orders.test"; // no-mistakes-disable-line no-import-only-test-files -- explicit integration test manifest required by this runner
```

## Related rules

- [`vitest-mock-test-file-naming`](vitest-mock-test-file-naming.md) keeps mock
  test ownership visible in filenames.
