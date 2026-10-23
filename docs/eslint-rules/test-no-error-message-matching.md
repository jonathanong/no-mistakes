# `no-mistakes/test-no-error-message-matching`

## Why

Error text changes frequently and is often user-facing copy. Tests should prove
the stable error type, code, or structured behavior instead.

## Disallowed

```ts
expect(error.message).toContain("user not found");
```

## Allowed

```ts
expect(error).toBeInstanceOf(NotFoundError);
expect(error.code).toBe("USER_NOT_FOUND");
```

## Options

This rule has no options.

## Fix

Assert an error class, code, status, or other contract field rather than the
message string.

## Limits

A renamed `expect`, a namespace member, a string or expression-free template key, optional chaining, a comma callee, and one `const` alias are the same assertion. `err.body.message` stays allowed. A computed identifier key is not a static name. A module-scope call named `expect` is checked whatever its source is, including `@playwright/test`, `./setup`, `bun:test`, `chai`, `const { expect } = require("vitest")`, and a local `const`, `let`, or `function` named `expect`. A parameter or other nested-scope binding named `expect` is not. A string key `["call"]` forwards like `.call`, including `expect(err.message).toMatch["call"](null, "x")`. A CommonJS program wrapper is still module scope.

`expect` brought in through another file is not visible here.

An immutable local alias of a proven module-level expectation remains the same
assertion: `const local = check; local(error.message).toEqual("missing")` inside
a function is reported when `check` imports Vitest's `expect`. A local alias of
a parameter or an unrelated function is not that binding. A nested binding
named `expect` is followed only when its constant initializer proves that
identity; its spelling alone is insufficient.

```ts
import { check } from "./barrel";
check(error.message).toMatch("missing");
```

`barrel` may be `export { expect as check } from "vitest"` or `export * from "vitest"`.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/test-no-error-message-matching -- localization regression explicitly owns this copy
expect(error.message).toBe("Translated error");
```

## Related rules

- [`test-no-shared-state`](test-no-shared-state.md) keeps tests isolated.
