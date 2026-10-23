# `no-mistakes/no-delete-property`

## Why

Deleting a property changes object shape in place and makes data flow harder to
trace than an explicit immutable omission or nullable value.

## Disallowed

```ts
delete user.token;
```

## Allowed

```ts
const { token: _token, ...publicUser } = user;
const clearedUser = { ...user, token: null };
```

## Options

This rule has no options.

## Limits

Every property deletion is checked directly, including aliases, static string
or expression-free template keys, optional members, and dynamic keys:
`delete rows[key]` is a property deletion regardless of the key's value.
The rule does not infer deletion inside called functions: `remove.call(null,
rows)` and an imported `remove(rows)` do not prove a deletion from this file.

## Fix

Create an omitted copy when the field must disappear, or assign an explicit
sentinel when the shape must remain stable.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/no-delete-property -- cleanup is required by this mutable third-party object
delete externalCache[key];
```

## Related rules

- [`no-object-mutation`](no-object-mutation.md) reports other shared-object writes.
- [`ts-preserve-null-option-defaults`](ts-preserve-null-option-defaults.md)
  preserves explicit null semantics.
