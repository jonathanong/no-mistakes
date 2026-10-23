# `no-mistakes/ts-no-function-aliases`

## Why

An extra forwarding name obscures the original callable. The rule preserves
released identifier-alias findings, while requiring a stronger static rewrite
proof for the member and forwarding forms added in #2516. Private receivers and
runtime receiver expressions cannot be replaced by calling a public original.

## Disallowed

```ts
import { createUser } from "./users";
export const makeUser = input => createUser(input);
```

Both paths require unchanged arguments in parameter order and a reachable target:
an imported binding, a static member path rooted at an import, or an exported
module-level function or const in the same file. A default-expression export
(`export default original`) proves reachability only while its binding is
unchanged; named exports retain their live binding.

Static strings, expression-free template keys, optional calls, and proven
`.call` / literal-array `.apply` forwarding retain the imported target:

```ts
import * as users from "./users";
const makeUser = input => users["createUser"](input);
const forwarded = input => users.createUser.call(users, input);
const applied = input => users.createUser.apply(users, [input]);
```

### Released identifier compatibility

For a raw identifier callee, the rule preserves v0.95.0 reporting for reachable
imports and same-file exports. Parameter/return annotations, annotated bindings,
async and generator functions, statement-only calls, predicates, JSDoc contracts,
wrapper generics and generic call arguments do not suppress those findings:

```ts
import { original } from "./api";
export function typed(value: string): string { return original(value); }
export const typedArrow = (value: Foo) => original(value);
export async function asynchronous(value) { return original(value); }
export function discarded(value) { original(value); }
export const contextual: F = value => original(value);
const specialized = value => original<string>(value);
```

This compatibility exception retains historical diagnostics without claiming
that their declared or return contracts are equivalent. Reachability, unchanged
arguments, recursion guards and suppressions still apply. Private identifiers
remain excluded. The exception uses the original callee syntax; normalizing a
`.call` or `.apply` target to an identifier never grants compatibility.

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
Expanded member paths must preserve the wrapper's contract, so predicates and
unproved declared signatures remain allowed on those paths:

```ts
import * as users from "./users";
import { createUser, LIST } from "./users";
export function narrowed(value: unknown): value is Code {
  return users.isCode(value);
}
export const includes = value => (LIST as readonly string[]).includes(value);
export const normalized = value => createUser(normalize(value));
export const reordered = (first, second) => createUser(second, first);
```

Direct and forwarded self-recursion are allowed.

## Limits

The stronger contract checks apply to expanded member, assertion and forwarding
paths. Imported signatures are unavailable to this file-local rule, so an
annotated wrapper over an imported member or `.call` / `.apply` target remains
unproved. A raw imported identifier retains the compatibility behavior above.

For a same-file exported callable reached through forwarding, a declared wrapper
signature requires matching parameter kinds and optionality, matching annotation
presence on every parameter, and identical primitive keywords where annotations
are present. Type references, structural types, predicates, overloads and generic
signatures remain unproved. Attached JSDoc contracts, contextual bindings and
assignment-based wrappers are also excluded from expanded-path findings.

Expanded async/generator wrappers change the call result contract and are
excluded. A statement-only expanded call drops its result and is checked only
when a same-file exported target explicitly returns `void`.

Raw asserted callees/results remain outside identifier compatibility, including
`original!(value)`, `original(value)!` and `await original(value)!`. An assertion
on an explicit `.call` / `.apply` receiver is excluded too. Generic call arguments
on an identifier (`original<string>(value)`) retain released behavior; an
instantiated callee expression (`(original<string>)(value)`) does not. Explicit
generic arguments on expanded calls remain unproved.

Computed identifier members, private same-file aliases and cross-file re-exports
do not establish a reachable static path. For example, `export * from "./users"`
does not create a local callable binding, and `users[name](value)` does not prove
which member is called. Defaults and destructured parameters do not establish an
unchanged parameter list. Template-key assignment wrappers, added in #2516,
retain the expanded-path checks even when their body calls an identifier.
`.apply` requires a literal array; a changed receiver or arguments are not aliases.
Type-only imports are not runtime callees. A method literally named `call` or
`apply` on an imported namespace is an actual export; on other imported values
those names can be Function intrinsics, so only proven forwarding is checked.
The rule does not resolve another file to compare signatures.

## Options

This rule has no options.

## Fix

Export or call the reachable original directly. For expanded forms, preserve its
receiver, arguments and contract. Keep a facade when it exposes a private
implementation or transforms inputs. A justified compatibility facade can use
suppression rather than silently changing released identifier-rule behavior.

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
