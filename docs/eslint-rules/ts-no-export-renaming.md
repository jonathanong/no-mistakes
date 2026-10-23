# `no-mistakes/ts-no-export-renaming`

## Why

Value-export renames create an extra symbol identity that obscures callers and
weakens structural code search.

## Disallowed

```ts
const createUser = () => {};
export { createUser as makeUser };
```

## Allowed

```ts
export const createUser = () => {};
```

## Options

- `allowDefaultReExports` permits default re-exports; it defaults to `false`.
- `includePathPatterns` scopes files to check; it defaults to all files.

## Limits

This rule checks value-export syntax, including explicit cross-file renames
such as `export { read as other } from "./db"`. It does not inspect the target
module: `export * from "./db"` supplies no rename to judge. Type-only exports,
such as `export type { Input as OtherInput } from "./types"`, remain allowed;
call aliases, member keys, and forwarding wrappers are outside this rule.

## Fix

Export the original value name, or rename the declaration itself so source and
public API use the same symbol.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/ts-no-export-renaming -- public compatibility alias
export { createUser as makeUser };
```

## Related rules

- [`ts-no-function-aliases`](ts-no-function-aliases.md) rejects implementation
  wrappers that create the same indirection.
