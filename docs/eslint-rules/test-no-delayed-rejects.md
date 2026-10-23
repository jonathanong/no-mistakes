# `no-mistakes/test-no-delayed-rejects`

## Why

A promise can reject while a test coordinates another operation. Attach the
rejection observer before that coordination so a runner never sees the expected
rejection as unhandled.

## Disallowed

```ts
async function rejectsWhenLocked() {
  const update = updateCommunityAgentPrompt(input);
  await releaseLock();
  await expect(update).rejects.toMatchObject({ status: 403 });
}
```

The rule also recognizes the literal computed form and clean `expect.soft`:

```ts
async function rejectsWhenLockedSoft() {
  const update = updateCommunityAgentPrompt(input);
  await releaseLock();
  await expect.soft(update)["rejects"].toThrow();
}
```

An immutable promise alias does not restart the rejection window. The promise
can reject before the alias is declared:

```ts
async function rejectsThroughAlias() {
  const update = updateCommunityAgentPrompt(input);
  await releaseLock();
  const pending = update;
  await expect(pending).rejects.toThrow();
}
```

## Allowed

Capture the rejection immediately, then assert the non-rejecting capture after
coordination completes:

```ts
async function observesBeforeCoordination() {
  const update = updateCommunityAgentPrompt(input);
  const rejection = update.catch((error: unknown) => error);
  await releaseLock();
  await expect(rejection).resolves.toMatchObject({ status: 403 });
}
```

An assertion attached before any coordination await is also allowed:

```ts
async function assertsImmediately() {
  const update = updateCommunityAgentPrompt(input);
  await expect(update).rejects.toThrow();
}
```

Directly awaiting the promise or passing it directly in an array to built-in
`Promise.allSettled` also observes its rejection. The built-in `Promise.all`,
`any`, and `race` are recognized when their aggregate promise is immediately
awaited; otherwise that promise can become the new unhandled rejection.

The same promise, through its original binding or an immutable alias, may be
asserted later when it first receives an
unconditional, structurally non-rejecting `catch` or rejection-side `then`
handler. The recognized handler may return a literal, `void 0`, or nothing.
Returning the rejection reason, an identifier named `undefined`, or the result
of calling other code is not assumed safe because the discarded child promise
can itself reject unhandled.

## Scope

The rule recognizes global `expect` plus a named `expect` import from `vitest`
or `@jest/globals`. It follows an immutable `const` promise binding through
same-file `const` identity aliases used as the single `expect` argument within
the same function or at module scope (top-level `await`). TypeScript `as` and
non-null expressions around an alias initializer preserve that identity.
Aliases declared after a coordination await retain the original promise's
creation time. A proven rejection observer attached through any alias protects
the same promise. The rule does not infer identity through destructuring,
mutable or reassigned bindings, conditional values, or cross-function flows;
attach the rejection observer immediately in those forms.

In script mode, a `with` object can intercept an outer binding read. An alias
whose initializer or later reference crosses that object scope is not treated
as the outer promise. Lexical promise and alias bindings declared inside the
`with` body remain analyzable.

Module-level delayed rejects are reported the same way as inside a function:

```ts
const update = updateCommunityAgentPrompt(input);
await releaseLock();
await expect(update).rejects.toMatchObject({ status: 403 });
```

The rule stays silent when the observer itself is attached before coordination,
including a stored matcher promise:

```ts
const matcher = expect(update).rejects.toThrow();
await releaseLock();
await matcher;
```

That form is still rejected by the Jest or Vitest `valid-expect` rule because
the async matcher was not awaited at its creation.

## Options

This rule has no options.

## Fix

There is no autofix. Attach a rejection handler to the original promise or any
proven alias before the first coordination await, then assert its captured value
after coordination. A `for await` loop,
an async-generator `yield`, and asynchronous disposal when an `await using`
scope exits are also intervening suspensions. Do not store an un-awaited
`expect(...).rejects` matcher promise; the test runner's `valid-expect` rule
owns that separate error.

## Limits

A renamed `expect`, a same-file immutable alias (including inside a function),
`v.expect`, and `expect(promise)["rejects"]` or an expression-free template key
are the same observer. `.call(null, promise)` and `.apply(null, [promise])`
forward that promise argument, including optional calls and static `"call"` /
expression-free template keys. An earlier forwarded matcher also attaches its
observer before a later coordination await.

Nested forwarding uses the function actually passed as the receiver:
`expect.call.call(expect, null, promise)` observes the promise, while
`expect.call.call(other, null, promise)` invokes `other` and cannot suppress a
later delayed `expect(promise).rejects` finding. An extracted intrinsic such as
`const invoke = expect.call` is not itself an expectation binding.

A computed identifier key is not static. `expect.poll` receives a callback,
including through an alias or forwarding call, and is not `expect(promise)`.
`expect.call(promise)` passes no expectation argument; `.apply` with an unknown
or spread argument list cannot establish which promise is observed.

`expect` brought in through another file is not visible here.

```ts
import { check } from "./barrel";
await check(update).rejects.toThrow();
```

`barrel` may be `export { expect as check } from "vitest"`.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/test-no-delayed-rejects -- operation cannot reject before this barrier
await expect(update).rejects.toThrow();
```

## Related rules

- The Jest or Vitest `valid-expect` rule requires async matcher promises to be
  awaited.
- [`async-call-disposition`](async-call-disposition.md) requires configured
  production async calls to be explicitly handled.
