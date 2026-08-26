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

## Fix

Remove the unnecessary `await`, or collect async work with `map` and await
`Promise.all` (or use an explicit sequential loop).

## Suppression

```ts
// eslint-disable-next-line no-mistakes/await-array-methods -- compatibility code; no async callback
await values.forEach(record);
```

## Related rules

- [`async-call-disposition`](async-call-disposition.md) for explicit handling
  of configured promises.
