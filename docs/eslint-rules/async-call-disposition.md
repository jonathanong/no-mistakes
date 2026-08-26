# `no-mistakes/async-call-disposition`

## Why

Configured async boundaries, such as enqueue or scheduling helpers, must make
their promise disposition explicit. A bare call can silently lose a rejection.

## Disallowed

```ts
enqueueEmail(user.id);
```

## Allowed

```ts
await enqueueEmail(user.id);
return enqueueEmail(user.id);
void enqueueEmail(user.id); // intentionally fire and forget
await Promise.all([enqueueEmail(first), enqueueEmail(second)]);
```

## Options

- `targets` is an array of target groups. Each group may set
  `sourceSpecifierPatterns` and `calleeNamePatterns`; values are glob or
  `/regex/` strings. Only calls matching a configured group are checked.

## Fix

Await or return work whose failure belongs to the caller. Use `void` only when
the call is deliberately detached.

## Editor suggestion

For a bare configured call, ESLint offers one suggestion: prefix it with
`void`. It does not guess whether `await`, `return`, or detachment is correct.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/async-call-disposition -- queue is intentionally best effort
enqueueEmail(user.id);
```

## Related rules

- [`async-try-catch-return-await`](async-try-catch-return-await.md) for returned
  promises inside protected error boundaries.
