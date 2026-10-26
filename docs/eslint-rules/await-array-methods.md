# `no-mistakes/await-array-methods`

## Why

Array methods such as `sort` return synchronously. Awaiting them suggests that
the method or its callback work was asynchronous when it was not.

## Disallowed

```ts
const users = [];

await users.sort();
```

## Allowed

```ts
const users = [];

await Promise.all(users.map((user) => sendEmail(user)));
users.sort(compareUsers);
```

## Options

This rule has no options.

A string or expression-free template method name, an optional call, a single `const` alias of the array, and `.call` or a literal-array `.apply` are the same method.

## Fix

Remove the unnecessary `await`, or collect async work with `map` and await
`Promise.all` (or use an explicit sequential loop).

## Limits

The checked methods are `sort`, `toSorted`, `every`, `findIndex`, `slice`, and
`toSpliced`. The receiver must be a known array literal or a stable `const`
alias, so an unknown parameter such as `users` is outside the rule's evidence.

`const { sort } = items; await sort()` is not the same call: destructuring a method drops the receiver. `.apply` with a non-literal argument list is not rewritten. `const alias = items as number[]` is still `items`. A shadowed spelling does not hide that array: `const values = []; { const alias = values; { const values = alias; await values.sort(); } }` is still the outer array.

## Suppression

```ts
const values = [2, 1];

// eslint-disable-next-line no-mistakes/await-array-methods -- compatibility code; no async callback
await values.sort();
```

## Related rules

- [`async-call-disposition`](async-call-disposition.md) for explicit handling
  of configured promises.
