# `no-mistakes/ts-no-export-renaming`

## Why

Value-export renames create an extra symbol identity that obscures callers and
weakens structural code search.

## Disallowed

```ts
const createUser = () => {};
export { createUser as makeUser };
export import makeUser = createUser;
exports.makeUser = createUser;
module.exports.makeUser = createUser;
```

## Allowed

```ts
export const createUser = () => {};
export * from "./accounts";
declare function createUser(): void;
export import createUser = createUser;
exports.createUser = createUser;
```

## Options

- `allowDefaultReExports` permits default re-exports; it defaults to `false`.
- `includePathPatterns` scopes files to check; it defaults to all files.
- `unanalyzable` is `"finding"` or `"ignore"`. It defaults to `"finding"`.
  An `export import`, `exports.Name`, or `module.exports.Name` write whose
  right-hand side is not one identifier is reported instead of passing
  silently. `"ignore"` reports only definite renames.

## Limits

This rule checks value-export syntax, including explicit cross-file renames
such as `export { read as other } from "./db"`. It does not inspect the target
module: `export * from "./accounts"` supplies no rename to judge. Type-only exports,
such as `export type { Input as OtherInput } from "./types"`, remain allowed;
call aliases, member keys, and forwarding wrappers are outside this rule.

`export import makeUser = createUser` is a rename when the names differ. The
same name stays quiet. `exports.makeUser = createUser` and
`module.exports.makeUser = createUser` are the same rename when `exports` and
`module` are the CommonJS bindings, not locals. A qualified name, a call, a
`require()` module reference, or a computed name that is not a string is not
one identifier, so it is `unanalyzable` instead of a silent pass. Assigning
`module.exports` itself is not a named rename.

## Fix

Export the original value name, or rename the declaration itself so source and
public API use the same symbol.

## Suppression

`no-mistakes-disable-file` must be the first token in the file. The three comments below are separate forms.

```ts
// no-mistakes-disable-file no-mistakes/ts-no-export-renaming -- public compatibility aliases for this module
```

```ts
// no-mistakes-disable-next-line no-mistakes/ts-no-export-renaming -- public compatibility alias
export import makeUser = createUser;
```

```ts
exports.publishUser = createUser; // no-mistakes-disable-line no-mistakes/ts-no-export-renaming -- public compatibility alias
```

Standard ESLint and Oxlint disable comments also apply.

## Related rules

- [`ts-no-function-aliases`](ts-no-function-aliases.md) rejects implementation
  wrappers that create the same indirection.
