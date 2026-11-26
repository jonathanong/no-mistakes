# `no-mistakes/ts-no-const-aliases`

## Why

A differently named `const` alias adds no behavior while hiding the original
symbol from readers and static analysis.

## Disallowed

```ts
const createAccount = createUser;
export const publicCreateAccount = createUser;
import createAccount = createUser;
```

## Allowed

```ts
const createAccount = (input: Input) => createUser(input);
const accountId = user.id;
const settings: Config = incoming;
const assertedSettings = incoming as Config;
const checkedSettings = incoming satisfies Config;
import createUser = createUser;
```

## Options

- `unanalyzable` is `"finding"` or `"ignore"`. It defaults to `"finding"`.
  A value import-equals whose right-hand side is not one identifier
  (`accounts.createUser`, `require("./accounts")`) is reported instead of
  passing silently. `"ignore"` reports only definite aliases.

## Limits

This rule checks differently named identifier initializers in `const`
declarations without explicit type intent. Type-annotated identifier bindings,
`as` and angle-bracket assertions, and `satisfies` expressions are allowed,
including through parentheses and non-null wrappers. Plain aliases wrapped only
in parentheses or non-null assertions remain reported. Member reads,
destructuring, calls, and their `.call` / `.apply` wrappers do not establish an
identifier alias: `const property = db["read"]` stays allowed. It does not resolve
another file's exports or infer whether a value is a function.

A value import-equals is the same alias when the names differ:
`import createAccount = createUser`. The same name,
`import createUser = createUser`, stays quiet. A qualified name or a
`require()` module reference is not one identifier, so it is `unanalyzable`
instead of a silent pass. `export import` is an export rename and is judged by
[`ts-no-export-renaming`](ts-no-export-renaming.md), not by this rule. `as` and
`satisfies` const initializers stay allowed.

## Fix

Use the original value name directly, or keep the declaration only when it
performs a meaningful transformation.

## Suppression

`no-mistakes-disable-file` must be the first token in the file. The three comments below are separate forms.

```ts
// no-mistakes-disable-file no-mistakes/ts-no-const-aliases -- compatibility names for this module
```

```ts
// no-mistakes-disable-next-line no-mistakes/ts-no-const-aliases -- stable compatibility name
import createAccount = createUser;
```

```ts
import makeAccount = createUser; // no-mistakes-disable-line no-mistakes/ts-no-const-aliases -- stable compatibility name
```

Standard ESLint and Oxlint disable comments also apply.

## Related rules

- [`ts-no-export-renaming`](ts-no-export-renaming.md) rejects renamed value
  exports.
- [`ts-no-function-aliases`](ts-no-function-aliases.md) rejects wrappers that
  only forward function calls.
