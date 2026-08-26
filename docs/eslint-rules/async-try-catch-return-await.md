# `no-mistakes/async-try-catch-return-await`

## Why

Returning a configured promise directly from a `try` skips that `try`'s local
`catch` when the promise later rejects. `return await` keeps the rejection in
the intended error boundary.

## Disallowed

```ts
try {
  return enqueueEmail(user.id);
} catch (error) {
  report(error);
}
```

## Allowed

```ts
try {
  return await enqueueEmail(user.id);
} catch (error) {
  report(error);
}
```

## Options

- `handlers` is an array of target groups. Each group may set
  `sourceSpecifierPatterns` and `calleeNamePatterns`; values are glob or
  `/regex/` strings.

## Fix

Use `return await` when the enclosing `catch` owns the rejection. Move the
call outside the `try` when that boundary is not meant to handle it.

## Editor suggestion

For a matching direct return, ESLint offers one suggestion: insert `await`
before the returned call. Review the error-boundary intent before applying it.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/async-try-catch-return-await -- rejection is handled by the caller
return enqueueEmail(user.id);
```

## Related rules

- [`async-call-disposition`](async-call-disposition.md) for all configured
  async calls, including non-returned calls.
