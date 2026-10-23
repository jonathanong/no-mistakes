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

Static member calls, expression-free template assignments, and proven
`.call` / literal-array `.apply` forwarding are aliases too:

```ts
const aliases = {};
aliases[`makeUser`] = (input: Input) => db?.createUser.call(db, input);
const optionalMemberCall = (input: Input) => db.createUser?.(input);
const rpcCall = (input: Input) => rpc.call(input);
const rpcApply = (input: Input) => rpc.apply(input);
export function makeAnotherUser(input: Input) {
  return db["createUser"](input);
}
```

## Allowed

```ts
export { createUser };
export function makeUser(input: Input) {
  return createUser(normalize(input));
}

const api = {};
api.method = (input: Input) => api.method(input); // Direct self-recursion is not an alias.
function recursiveCall(input: Input) {
  return recursiveCall.call(undefined, input); // Normalized self-recursion is not an alias.
}
export function makeAuditedUser(input: Input) {
  audit(input);
  return db.createUser(input);
}
```

## Limits

The rule reports a member call only when its property is static. Computed names
that depend on a runtime value, dynamic `this` calls, and `.apply` calls whose
argument list is not a literal array stay unmatched:

```ts
export const dynamic = (input: Input, key: string) => db[key](input);
export function dynamicThis(input: Input) {
  return this(input);
}
export const opaqueApply = (...args: Input[]) => createUser.apply(undefined, args);
```

Forwarding must preserve argument order, rest spreading, and the method
receiver. Defaults, changed arguments, or a different `.call` / `.apply`
receiver mean the wrapper has behavior the rule cannot prove is an alias.
Optional invocation is checked when the underlying member name is static.
Methods literally named `call` or `apply` are also checked as ordinary member
calls when their own arguments match the wrapper; the forwarding interpretation
is considered separately when its receiver and arguments can be proven.

## Options

This rule has no options.

## Fix

Export or call the original function directly. Keep a wrapper only when it
performs meaningful transformation, validation, or policy.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/ts-no-function-aliases -- stable deprecation facade
export function makeUser(input: Input) {
  return createUser(input);
}
```

## Related rules

- [`ts-no-export-renaming`](ts-no-export-renaming.md) rejects renamed value
  exports.
