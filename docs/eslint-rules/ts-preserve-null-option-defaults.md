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

## Fix

Test explicitly for `undefined` and preserve `null`; do not use `??`, `||`,
`??=`, or `||=` for a nullable option member.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/ts-preserve-null-option-defaults -- API contract deliberately treats null as omission
return options.label ?? "Untitled";
```

## Related rules

- [`react-no-nullish-react-node`](react-no-nullish-react-node.md) applies the
  same rule to ReactNode fallbacks.
