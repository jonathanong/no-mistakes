# `no-mistakes/ts-no-function-aliases`

## Why

A function that only calls another function under a new name adds no behavior
while hiding the real implementation from readers and analysis.

## Disallowed

```ts
export function makeUser(input: Input) {
  return createUser(input);
}
```

## Allowed

```ts
export { createUser };
export function makeUser(input: Input) {
  return createUser(normalize(input));
}
```

## Options

This rule has no options.

## Fix

Export or call the original function directly. Keep a wrapper only when it
performs meaningful transformation, validation, or policy.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/ts-no-function-aliases -- stable deprecation facade
export function makeUser(input: Input) { return createUser(input); }
```

## Related rules

- [`ts-no-export-renaming`](ts-no-export-renaming.md) rejects renamed value
  exports.
