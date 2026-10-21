# `no-mistakes/await-array-methods`

## Why

Array iteration helpers return synchronously. Awaiting them suggests that the
callback work was awaited when it was not.

## Disallowed

```ts
await users.forEach(async (user) => sendEmail(user));
```

## Allowed

```ts
await Promise.all(users.map((user) => sendEmail(user)));
users.forEach(logUser);
```

## Options

This rule has no options.

A string or expression-free template method name, an optional call, a single `const` alias of the array, and `.call` or a literal-array `.apply` are the same method.

## Fix

Remove the unnecessary `await`, or collect async work with `map` and await
`Promise.all` (or use an explicit sequential loop).

## Limits

`const { sort } = items; await sort()` is not the same call: destructuring a method drops the receiver. `.apply` with a non-literal argument list is not rewritten. `const alias = items as number[]` is still `items`.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/await-array-methods -- compatibility code; no async callback
await values.forEach(record);
```

## Related rules

- [`async-call-disposition`](async-call-disposition.md) for explicit handling
  of configured promises.
