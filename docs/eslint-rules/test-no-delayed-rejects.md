# `no-mistakes/test-no-delayed-rejects`

## Why

A promise can reject while a test coordinates another operation. Attach the
rejection observer before that coordination so a runner never sees the expected
rejection as unhandled.

## Disallowed

```ts
const update = updateCommunityAgentPrompt(input);
await releaseLock();
await expect(update).rejects.toMatchObject({ status: 403 });
```

The rule also recognizes the literal computed form and clean `expect.soft`:

```ts
const update = updateCommunityAgentPrompt(input);
await releaseLock();
await expect.soft(update)["rejects"].toThrow();
```

## Allowed

Capture the rejection immediately, then assert the non-rejecting capture after
coordination completes:

```ts
const update = updateCommunityAgentPrompt(input);
const rejection = update.catch((error: unknown) => error);
await releaseLock();
await expect(rejection).resolves.toMatchObject({ status: 403 });
```

An assertion attached before any coordination await is also allowed:

```ts
const update = updateCommunityAgentPrompt(input);
await expect(update).rejects.toThrow();
```

The same direct binding may be asserted later when it first receives an
unconditional, structurally non-rejecting `catch` or rejection-side `then`
handler. The recognized handler may return a literal, `void 0`, or nothing.
Returning the rejection reason, an identifier named `undefined`, or the result
of calling other code is not assumed safe because the discarded child promise
can itself reject unhandled.

## Scope

The rule recognizes global `expect` plus a named `expect` import from `vitest`
or `@jest/globals`. It follows only one immutable, non-aliased `const` binding
used directly as the single `expect` argument within the same function. It does
not infer promise identity through aliases, destructuring, mutable bindings, or
cross-function flows; attach the rejection observer immediately in those forms.

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

There is no autofix. Attach a rejection handler before the first coordination
await, then assert its captured value after coordination. A `for await` loop and
an async-generator `yield` are also intervening suspensions. Do not store an
un-awaited `expect(...).rejects` matcher promise; the test runner's
`valid-expect` rule owns that separate error.

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
