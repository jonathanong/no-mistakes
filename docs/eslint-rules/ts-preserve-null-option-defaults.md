# `no-mistakes/ts-preserve-null-option-defaults`

## Why

When an option type permits `null`, explicit null often has different meaning
from omission. `??`, `||`, and their assignments collapse that distinction.

## Disallowed

```ts
function render(options: { label?: string | null }) {
  return options.label ?? "Untitled";
}
```

## Allowed

```ts
function render(options: { label?: string | null }) {
  return options.label === undefined ? "Untitled" : options.label;
}
```

## Options

- `includePathPatterns` and `excludePathPatterns` scope files.
- `optionObjectNames` lists option variable names.
- `optionObjectNamePatterns` lists name patterns.

All lists default to empty. Object-destructuring defaults remain allowed because
they apply only to `undefined`.

## Limits

`` options[`label`] `` and `` const { [`label`]: text } = options `` are `options.label`. A computed identifier key is not:
`options[key] ?? "Untitled"` does not prove which nullable member is read.
Same-file object aliases and direct static destructuring patterns preserve the option
member. A parameter-default destructure such as
`function render({ label } = options) { return label ?? "Untitled"; }` does not
prove that the caller supplied that option object. Imported type definitions
are not resolved: `import type { Options } from "./types"` does not prove which
members admit null. Call forwarding wrappers do not change these type proofs.

Plain `const` scalar aliases also preserve a known nullable member, including
chains and reads from an enclosing block:

```ts
function render(options: { label?: string | null }) {
  const { label } = options;
  const alias = label;
  const chained = alias;
  return chained ?? "Untitled"; // Reports: this still discards explicit null.
}
```

The new scalar path requires a plain identifier initializer and an already
completed, unwritten `const` source binding. It uses the actual lexical binding,
so a nearer hoisted variable or later declaration does not borrow an outer
nullable fact. `let`/`var` sources or aliases, assignment aliases, TypeScript
wrappers around scalar aliases, imported values and forward reads are outside
this added proof. Existing direct-member and object-alias behavior is unchanged.

```ts
function render(options: { label?: string | null }) {
  const label = options.label;
  let alias = label;
  return alias ?? "Untitled"; // Mutable scalar aliases are not followed.
}
```

## Fix

Test explicitly for `undefined` and preserve `null`; do not use `??`, `||`,
`??=`, or `||=` for a nullable option member. The same rewrite applies after
extracting the member, destructuring it, copying it through const aliases, or
reading the const from an enclosing block:

```ts
function render(options: { label?: string | null }) {
  const { label } = options;
  const alias = label;
  const chained = alias;
  return chained === undefined ? "Untitled" : chained;
}
```

For an `alias || fallback` finding, this explicit check also preserves other
falsy values such as an empty string. Keep the fallback in the conditional arm
so it is evaluated only for `undefined`. There is no automatic fix.

## Suppression

```ts
// no-mistakes-disable-next-line ts-preserve-null-option-defaults -- API contract deliberately treats null as omission
return options.label ?? "Untitled";
```

The rule also supports `no-mistakes-disable-line` and a leading
`no-mistakes-disable-file`, with a specific rule name or no rule list. Standard
ESLint suppression comments remain supported. Suppression hides a finding without
discarding the nullable facts used by later expressions.

## Related rules

- [`react-no-nullish-react-node`](react-no-nullish-react-node.md) applies the
  same rule to ReactNode fallbacks.
