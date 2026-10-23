# `no-mistakes/ts-no-function-aliases`

## Why

An alias wrapper adds an extra name only when its callers can already reach the
same callable with the same arguments and contract. A facade over a private
receiver, or a wrapper that adds a type contract, cannot be replaced that way.

## Disallowed

```ts
import { createUser } from "./users";
export const makeUser = input => createUser(input);
```

The rule requires unchanged arguments in parameter order and a reachable target:
an imported binding, a static member path rooted at an import, or an exported
module-level function or const in the same file. A default-expression export
(`export default original`) proves this only while the binding is unchanged;
named exports retain their live binding. Static strings, expression-free
template keys, optional calls, and proven `.call` / literal-array `.apply`
forwarding retain that target:

```ts
import * as users from "./users";
const makeUser = input => users["createUser"](input);
const forwarded = input => users.createUser.call(users, input);
const applied = input => users.createUser.apply(users, [input]);

export function original(value: string): boolean {
  return value.length > 0;
}
const duplicate = (value: string): boolean => original(value);
```

## Allowed

A private receiver is an implementation detail unavailable to the wrapper's
callers. These facades have no direct public replacement:

```ts
const catalog = createCatalog();
export function isCode(value) {
  return catalog.isCode(value);
}
const pattern = /\bVALIDATE\b/i;
export const isValidation = value => pattern.test(value);
export const retrieve = id => getClient().refunds.retrieve(id);
```

Literal, call-result, `this`, local and asserted receivers are also excluded.
A type predicate, changed arguments, or an unproved declared signature preserves
meaningful behavior:

```ts
import { isCode, createUser, LIST } from "./users";
export function narrowed(value: unknown): value is Code {
  return isCode(value);
}
export const includes = value => (LIST as readonly string[]).includes(value);
export const normalized = value => createUser(normalize(value));
export const reordered = (first, second) => createUser(second, first);
```

Direct and forwarded self-recursion are allowed.

## Limits

Imported signatures are unavailable to this file-local rule. An imported target
is checked only for an unannotated wrapper; it does not prove that a declared
parameter type, optionality or return type is redundant:

```ts
import { convert } from "./convert";
export const typedFacade = (value: string): string => convert(value);
```

For a same-file exported function or function-valued const, declared wrapper
types are admitted only when corresponding declared types are identical
primitive keywords with matching parameter optionality. Type references,
structural types, predicates, overloads and generic signatures remain unproved.
Explicit generic call arguments, such as `convert<string>(value)`, also specialize
the contract and are excluded. An assertion on an explicit `.call` / `.apply`
receiver is excluded just like an asserted member receiver.
Attached JSDoc parameter, return, type, template and overload contracts also
exclude a wrapper; absent TypeScript AST annotations do not erase those contracts. Annotated
wrapper bindings and assignment-based wrappers can acquire contextual signatures
and are excluded. Defaults and destructured parameters do not establish an
unchanged ordered parameter list.

Async and generator wrappers change the call result contract and are excluded.
A statement-only call drops its result and is checked only when the same-file
exported target explicitly returns `void`.

Computed identifier members, assertion receivers, private same-file aliases,
and cross-file re-exports do not establish a reachable static path. For example,
`export * from "./users"` does not create a local callable binding, and
`users[name](value)` does not prove which member is called. `.apply` requires a
literal array; a different method receiver or changed arguments are not aliases.
Type-only imports are not runtime callees. A method literally named `call` or
`apply` on an imported namespace is an actual export; on other imported values
those names can be Function intrinsics, so only proven forwarding is checked.
The rule does not resolve another file to compare signatures.

## Options

This rule has no options.

## Fix

Export or call the reachable original directly when the wrapper has the same
arguments and contract. Keep a facade when it exposes a private implementation,
validates or transforms inputs, or adds a meaningful type contract.

## Suppression

```ts
import { createUser } from "./users";
// no-mistakes-disable-next-line ts-no-function-aliases -- stable deprecation facade
export const makeUser = input => createUser(input);
```

`no-mistakes-disable-line` and `no-mistakes-disable-file` work the same way.
Standard ESLint/Oxlint disable directives also apply.

## Related rules

- [`ts-no-export-renaming`](ts-no-export-renaming.md) rejects renamed value
  exports.
