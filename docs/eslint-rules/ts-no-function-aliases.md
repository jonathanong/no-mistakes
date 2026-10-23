# `no-mistakes/ts-no-function-aliases`

## Why

An extra forwarding name obscures the original callable. For a bare identifier
callee, the rule preserves the exact v0.95.0 syntax-based decision. Member and
forwarding forms added in #2516 require stronger static rewrite proof. Private
receivers and runtime receiver expressions cannot be replaced by calling a
public original.

## Disallowed

```ts
import { createUser } from "./users";
export const makeUser = input => createUser(input);
```

Both paths require unchanged arguments in parameter order. A bare identifier
callee follows the released syntax rule regardless of its binding. Expanded
member and forwarding paths require a reachable target: an imported binding,
a static member path rooted at an import, or an exported module-level function
or const in the same file. A default-expression export (`export default original`)
proves expanded-path reachability only while its binding is unchanged; named
exports retain their live binding.

Static strings, expression-free template keys, optional calls, and proven
`.call` / literal-array `.apply` forwarding retain the imported target:

```ts
import * as users from "./users";
const makeUser = input => users["createUser"](input);
const forwarded = input => users.createUser.call(users, input);
const applied = input => users.createUser.apply(users, [input]);
```

### Released identifier compatibility

For any callee that becomes a bare identifier after removing `await`, optional
chain and non-null syntax, the rule preserves v0.95.0 reporting without checking
whether that identifier is imported, exported, private, global, destructured
from `require`, re-exported, or shadowed. Parameter/return annotations,
annotated bindings, async and generator functions, statement-only calls,
predicates, JSDoc contracts, wrapper generics and generic call arguments do not
suppress those findings:

```ts
import { original } from "./api";
export function typed(value: string): string { return original(value); }
export const typedArrow = (value: Foo) => original(value);
export async function asynchronous(value) { return original(value); }
export function discarded(value) { original(value); }
export const contextual: F = value => original(value);
const specialized = value => original<string>(value);
```

This compatibility path retains historical diagnostics without claiming that
the declared or return contracts are equivalent. The released naming, self-call,
and unchanged-argument checks still apply. Suppressions still apply. A bare
identifier that the released rule left silent stays silent; it does not fall
through to expanded analysis. Normalizing a `.call` or `.apply` target to an
identifier never grants compatibility.

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

The stronger contract checks apply to expanded member and forwarding
paths. Imported signatures are unavailable to this file-local rule, so an
annotated wrapper over an imported member or `.call` / `.apply` target remains
unproved. A bare identifier retains the compatibility behavior above.

For a same-file exported callable reached through forwarding, a declared wrapper
signature requires matching parameter kinds and optionality, matching annotation
presence on every parameter, and identical primitive keywords where annotations
are present. Type references, structural types, predicates, overloads and generic
signatures remain unproved. Attached JSDoc contracts, contextual bindings and
assignment-based wrappers are also excluded from expanded-path findings.

Expanded async/generator wrappers change the call result contract and are
excluded. A statement-only expanded call drops its result and is checked only
when a same-file exported target explicitly returns `void`.

Non-null callees/results retain released identifier behavior, including
`original!(value)`, `original(value)!`, `(await original(value))!`, and
`original?.(value)!`. An assertion on an explicit `.call` / `.apply` receiver
is excluded. Generic call arguments
on an identifier (`original<string>(value)`) retain released behavior; an
instantiated callee expression (`(original<string>)(value)`) does not. Explicit
generic arguments on expanded calls remain unproved.

Computed identifier members and cross-file re-exports do not establish a
reachable static member path. For example, `users[name](value)` does not prove
which member is called. A bare `remote(value)` follows released behavior even
when `remote` appears only in `export { remote } from "./users"`. Defaults and
destructured parameters do not establish an unchanged parameter list. Template
key assignment wrappers follow the released naming rule for bare identifier
callees.
`.apply` requires a literal array; a changed receiver or arguments are not aliases.
Forwarding must preserve argument order, rest spreading, and the method
receiver. Defaults, changed arguments, or a different `.call` / `.apply`
receiver mean the wrapper has behavior the rule cannot prove is an alias.
Optional invocation is checked when the underlying member name is static.
An `as` or angle-bracket cast around a callee is checked only when its type is
`typeof` that exact reachable callable, directly or through one same-file,
non-generic type alias. A cast around the returned call is checked only when
its primitive type is identical to an explicit return type on a same-file
exported callable. A single same-file type alias of that primitive is also
recognized. These proofs allow the wrapper to be removed
without discarding a distinct asserted type:

```ts
import { db } from "./db";
type Handler = typeof db.createUser;
const create = input => (db.createUser as Handler)(input);
// Fix: use db.createUser directly instead of create.

export function original(input: string): string { return input; }
type Output = string;
const forwarded = input => (original(input) as Output);
// Fix: use original directly instead of forwarded.
```

Unresolved or imported aliases, a changed primitive result type, and
`satisfies` contracts stay unreported on the expanded path. An
assertion on a receiver does not establish a reachable target. Recursion and
forwarding receiver checks still apply to casts that pass the type proof.
Methods literally named `call` or `apply` are also checked as ordinary member
calls when their own arguments match the wrapper; the forwarding interpretation
is considered separately when its receiver and arguments can be proven.
Type-only imports do not establish expanded-path reachability. A method
literally named `call` or `apply` on an imported namespace is an actual export;
on other imported values those names can be Function intrinsics, so only
proven forwarding is checked. The rule does not resolve another file to
compare signatures.

## Options

This rule has no options.

## Fix

Export or call the original directly when it is available. For expanded forms,
preserve its receiver, arguments and contract. Keep a facade when it exposes a
private implementation or transforms inputs. Suppress a justified bare-identifier
facade that the released rule reports.

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
