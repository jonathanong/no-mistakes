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

`unanalyzable` is `"finding"` or `"ignore"` and defaults to `"finding"`. An
assertion whose `expect` comes from a non-static `require` reports
`unanalyzable` when the compared value is an error message.
`check.call(null, err.message)` and `check.apply(null, [err.message])` report
that advisory when `check` is that unresolved expect. A `.call` spread that is not a literal array, and an `.apply` list that is not a literal array, are not that comparison. `check.call(null, ...[err.message])` still is. `require("vitest").expect.call(...[null, 42], err.message)` is not a message comparison. `check.apply(...[null, [42]], [err.message])` is not a message comparison. `check.apply(null, [...[err.message]])` still is. `check.call(null, err.message, ...extra)` and `check.apply(null, [err.message], ...extra)` still are, because the trailing spread does not replace the known message. `require("vitest").expect.apply(err.message, values)` is not a message comparison. `require("vitest").expect.apply(null, [err.message])` still is.

`import vt = require("vitest")`, `const { expect } = require("@playwright/test")`,
`require("vitest").expect`, and a Vitest namespace are expect bindings.
`require("vitest").test` is a runner, so a message comparison inside that
callback is the same diagnostic. Inside a test,
`err.message === ("boom" as string)`, `(v as typeof v).expect(err.message)`,
`v.expect.apply(null, [err.message] as const)`, and `(v.expect<string>)(err.message)`
are the same message comparison.

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

Playwright namespace and default-runner `.expect` members retain assertion
identity, including immutable namespace aliases. Vitest, Jest, and Playwright
runner imports and immutable same-file aliases establish the test body for
message equality and `.includes` checks. A Vitest default import used as a
namespace, such as `import vt from "vitest"; vt.test(...)`, establishes that
body. Calling that default import directly does not. `import vt from "vitest"; vt.expect(err.message)` is an expect binding. `import { test } from "vitest"; test.expect(err.message)` is not. Nested parameters and mutable aliases
do not establish runner identity. Assert the error type or code instead of its
message, regardless of runner or assertion spelling.

A renamed `expect`, a namespace member, a string or expression-free template key, optional chaining, a comma callee, and one `const` alias are the same assertion. `err.body.message` stays allowed. A computed identifier key is not a static name. A module-scope call named `expect` is checked whatever its source is, including `@playwright/test`, `./setup`, `bun:test`, `chai`, `const { expect } = require("vitest")`, and a local `const`, `let`, or `function` named `expect`. A parameter or other nested-scope binding named `expect` is not. A string key `["call"]` forwards like `.call`, including `expect(err.message).toMatch["call"](null, "x")`. A CommonJS program wrapper is still module scope.

`expect` brought in through another file is not visible here.

Captured-value analysis follows preceding immutable identifier declarations in
the same file. Mutable bindings, writes, forward reads, cycles, dynamic computed
keys, parameters, destructured bindings, helper returns, and object-container
reads do not prove that identity. Reads crossing a dynamic `with` scope or a
non-strict scope containing a potential direct `eval` call are not followed:
the latter can introduce a nearer binding that static scope resolution misses.
The added value analysis keeps the existing assertion and test-body boundaries;
it does not turn arbitrary string operations outside tests into findings.
Captured `err.body.message` values keep the existing body-message exemption.
For captured values, `as string`, `<string>`, non-null, optional-chain, and
comma-last wrappers preserve provenance. Erased `satisfies` wrappers preserve
both the captured value and the expectation identity, including
`const text = error.message satisfies string`. Keep that type check when
rewriting the capture to `error.code`. Other asserted types are outside the
captured-value proof.
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

In a non-strict CommonJS/script function, direct eval can insert a nearer
variable even when the call is inside a nested block. The captured-value proof
does not follow that dynamic environment:

```js
// fixture.cjs: no "use strict"
const captured = error.message;
const checkScript = () => {
  {
    eval("var captured = 'ordinary'");
  }
  expect(captured).toBe("ordinary"); // This reads the eval-created local value.
};
checkScript();
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

Calls rooted at `test` or `it` retain the existing conservative test-body
reporting, including parameters and nested local bindings with those names.
Those names alone do not prove framework identity, but they do not exempt a
message comparison that the rule already checked. For example,
`function run(test) { test("case", () => error.message === "expected"); }`
still reports. Use an error type or code comparison instead. Binding-aware
runner aliases add supported forms; a renamed parameter such as `check`
without a proven runner initializer stays unknown.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/test-no-error-message-matching -- localization regression explicitly owns this copy
expect(error.message).toBe("Translated error");
```

`no-mistakes-disable-line`, `no-mistakes-disable-next-line`, and
`no-mistakes-disable-file` also suppress this rule when the comment names
`test-no-error-message-matching` or
`no-mistakes/test-no-error-message-matching`. Directive-looking string
literals do not.

## Related rules

- [`test-no-shared-state`](test-no-shared-state.md) keeps tests isolated.
