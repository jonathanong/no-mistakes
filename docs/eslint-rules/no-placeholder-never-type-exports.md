# `no-mistakes/no-placeholder-never-type-exports`

## Why

Exported `never` aliases are often placeholders that advertise an API with no
usable value and hide a missing type design.

## Disallowed

```ts
export type UserResponse = never;
```

## Allowed

```ts
export interface UserResponse {
  id: string;
}
```

## Options

This rule has no options.

## Limits

This rule checks direct exported `never` declarations and explicit type exports
of same-file `never` aliases. It does not resolve imported type definitions:
`import type { Missing } from "./types"; export type { Missing };` cannot prove
that `Missing` is `never`. Cross-file type re-exports and `export type * from
"./types"` remain unresolved. Runtime call aliases, static member keys, and
forwarding wrappers do not apply to this type-declaration check.

## Fix

Define the real exported contract or remove the temporary export until the
surface exists.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/no-placeholder-never-type-exports -- intentional impossible state used by a generated compatibility declaration
export type ImpossibleLegacyState = never;
```

## Related rules

- [`ts-no-export-renaming`](ts-no-export-renaming.md) keeps exported value
  identities direct.
