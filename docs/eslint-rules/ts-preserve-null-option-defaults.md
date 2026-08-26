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
