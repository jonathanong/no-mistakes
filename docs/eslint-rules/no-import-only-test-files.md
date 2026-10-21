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

`require(\`./users.test\`)`, `require?.("./users.test")`, `require["call"](null, "./users.test")`, and `require.call(null, "./users.test")` name the same test file. A template with an expression, or a computed identifier key, does not.

A test file reached through another module's re-export is not visible here.

```js
require("./barrel");
```

`barrel.js` may be `module.exports = require("./users.test")`.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/no-import-only-test-files -- explicit integration test manifest required by this runner
import "./users.test";
```

## Related rules

- [`vitest-mock-test-file-naming`](vitest-mock-test-file-naming.md) keeps mock
  test ownership visible in filenames.
