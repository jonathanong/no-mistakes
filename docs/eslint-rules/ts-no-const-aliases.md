# `no-mistakes/ts-no-const-aliases`

## Why

A differently named `const` alias adds no behavior while hiding the original
symbol from readers and static analysis.

## Disallowed

```ts
const createAccount = createUser;
export const publicCreateAccount = createUser;
```

## Allowed

```ts
const createAccount = (input: Input) => createUser(input);
const accountId = user.id;
```

## Options

This rule has no options.

## Limits

This rule checks differently named identifier initializers in `const`
declarations, including erased TypeScript assertions. Member reads,
destructuring, calls, and their `.call` / `.apply` wrappers do not establish an
identifier alias: `const property = db["read"]` stays allowed. It does not resolve
another file's exports or infer whether a value is a function.

## Fix

Use the original value name directly, or keep the declaration only when it
performs a meaningful transformation.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/ts-no-const-aliases -- stable compatibility name
const createAccount = createUser;
```

## Related rules

- [`ts-no-export-renaming`](ts-no-export-renaming.md) rejects renamed value
  exports.
- [`ts-no-function-aliases`](ts-no-function-aliases.md) rejects wrappers that
  only forward function calls.
