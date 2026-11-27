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
of same-file `never` aliases. Other same-file aliases, interfaces, classes,
enums, and namespaces stay quiet, including declarations already exported
from the file. It does not read another module. These forms report
`unanalyzable`, because the rule cannot tell whether the exported type is
`never`:

```ts
export type { Missing } from "@example/types";
export type * from "@example/types";
import type { Missing } from "@example/types";
export type { Missing };
```

`export type Local = never` remains the placeholder finding. Runtime call
aliases, static member keys, and forwarding wrappers do not apply to this
type-declaration check.

## Fix

Define the real exported contract or remove the temporary export until the
surface exists.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/no-placeholder-never-type-exports -- intentional impossible state used by a generated compatibility declaration
export type ImpossibleLegacyState = never;
```

`no-mistakes-disable-line`, `no-mistakes-disable-next-line`, and
`no-mistakes-disable-file` also suppress this rule when the comment names it.
Directive-looking string literals do not.

## Related rules

- [`ts-no-export-renaming`](ts-no-export-renaming.md) keeps exported value
  identities direct.
