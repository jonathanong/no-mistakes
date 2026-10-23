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

An immutable captured message remains message-derived when it reaches an
assertion through a same-file `const` chain:

```ts
const captured = error.message;
const text = captured;
expect(text).toContain("user not found");
```

Capture a stable contract field at the original capture point and compare that
field's contract instead. Keep the aliases and control flow when they are still
needed:

```ts
const captured = error.code;
const code = captured;
expect(code).toBe("USER_NOT_FOUND");
```

This also repairs static-key and optional message captures, forwarded expectation
calls, and message comparisons inside tests: preserve optional access when
needed, capture `error.code` instead of `error.message`, and compare a stable code
instead of searching or comparing message text. Capturing the contract at the
same point preserves snapshot timing if the source object or binding changes
later. Remove a message capture only when it has no other uses. There is no
automatic fix because the correct error contract is application-specific.
If other code still needs the captured message, keep that capture and add a
separate code capture at the same point for the assertion. Retain initializer
side effects, forwarding receivers, other arguments, and required type wrappers.
For a receiver with side effects, such as `loadError().message`, evaluate
`loadError()` once into a local error object, then read any still-needed message
and the code from that object. Do not call the receiver expression a second time.

## Limits

A renamed `expect`, a namespace member, a string or expression-free template key, optional chaining, a comma callee, and one `const` alias are the same assertion. `err.body.message` stays allowed. A computed identifier key is not a static name. A module-scope call named `expect` is checked whatever its source is, including `@playwright/test`, `./setup`, `bun:test`, `chai`, `const { expect } = require("vitest")`, and a local `const`, `let`, or `function` named `expect`. A parameter or other nested-scope binding named `expect` is not. A string key `["call"]` forwards like `.call`, including `expect(err.message).toMatch["call"](null, "x")`. A CommonJS program wrapper is still module scope.

`expect` brought in through another file is not visible here.

Captured-value analysis follows preceding immutable identifier declarations in
the same file. Mutable bindings, writes, forward reads, cycles, dynamic computed
keys, parameters, destructured bindings, helper returns, and object-container
reads do not prove that identity. Reads crossing a dynamic `with` scope are not followed.
The added value analysis keeps the existing assertion and test-body boundaries;
it does not turn arbitrary string operations outside tests into findings.
Captured `err.body.message` values keep the existing body-message exemption.
For captured values, `as string`, `<string>`, non-null, optional-chain, and
comma-last wrappers preserve provenance. Other asserted types and `satisfies`
are outside the added proof; the released direct-member behavior is unchanged.
References that cross a function-declaration scope are also skipped because a
hoisted function can run before the capture is initialized. Expression callbacks
created after their captured values can still be checked.

```ts
import { text } from "./captured-error";
expect(text).toBe("missing"); // The value's origin is in another file.

const selected = error[key]; // `key` is not a static property name.
expect(selected).toBe("missing");

function check({ message: value } = error) {
  expect(value).toBe("missing"); // A parameter default does not prove each argument.
}
```

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
