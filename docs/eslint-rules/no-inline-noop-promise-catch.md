# `no-mistakes/no-inline-noop-promise-catch`

## Why

A `.catch(() => {})` satisfies catch-presence linters without handling the
rejection. Failed work then looks complete. This rule requires observable
handling — a named handler, logging, reporting, a transform, or a rethrow —
so swallowed business failures stay reviewable.

The rule intentionally recognizes `.catch` and two-argument `.then` by syntax;
it does not prove that the receiver is a native `Promise`. Use the path and
callee allowlists for known custom APIs.

## Disallowed

```ts
saveUser(input).catch(() => {});
saveUser(input).catch(() => undefined);
saveUser(input).catch(() => {
  return;
});
saveUser(input).then(onSaved, () => void 0);
saveUser(input).catch(() => {
  "ignored";
});
```

## Allowed

```ts
saveUser(input).catch(logError);
saveUser(input).catch(function ignoreLockReleaseError() {});
saveUser(input).catch((error) => {
  report(error);
});
saveUser(input).catch((error) => {
  throw error;
});
saveUser(input).catch(() => null);
saveUser(input).catch(function* () {});
saveUser(input).catch(async function* () {});
```

## Options

- `checkedPathPatterns` limits checking to matching files. An empty list
  checks every file.
- `allowedPathPatterns` excludes matching files.
- `allowedCalleeNamePatterns` skips a no-op catch when the originating call's
  identifier or member name matches, walking through `.then` / `.finally` /
  `.catch` chains. Use this for named cleanup APIs such as `releaseLock()`.

Values are glob or `/regex/` strings. Invalid regex patterns are ignored.

## Fix

Replace the inline no-op with a named handler, `onError`, logging, a returned
fallback, or a rethrow. Do not suppress a production swallow to keep a
catch-presence rule quiet.

A bare literal expression inside a block is also a no-op: it is not the same
as returning that literal as an intentional fallback. A locally shadowed
`undefined` is treated as a fallback value, and generator callbacks are
allowed because calling them produces an iterator rather than an ignored
`undefined` result.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/no-inline-noop-promise-catch -- best-effort cache warmup
warmup().catch(() => {});
```

## Related rules

- [`async-call-disposition`](async-call-disposition.md) treats `.catch` as
  explicit promise disposition; it does not inspect whether the callback
  handles the rejection.
- [`async-try-catch-return-await`](async-try-catch-return-await.md) keeps
  configured rejections inside a `try`/`catch` boundary.
