# `no-mistakes/playwright-require-exported-component-attribute`

## Why

An exported component is a reusable UI boundary. Requiring a selector attribute
in every returned JSX branch makes that boundary discoverable to coverage tools.

## Disallowed

```tsx
export function SaveButton() {
  return <button>Save</button>;
}
```

## Allowed

```tsx
export function SaveButton() {
  return <button data-pw="save-button">Save</button>;
}
```

## Options

- `attributes` defaults to `["data-pw"]`.
- `componentNamePattern`, `components`, and `ignoreComponents` choose exported
  components to check.
- `wrappers` recognizes configured component wrappers.
- `allowSpreadAttributes` defaults to `false` and controls whether a JSX spread
  can satisfy the attribute requirement.
- `exportTypes` limits named/default exports, and `checkAnonymousDefault`
  controls anonymous default components.

## Fix

Add one configured attribute to each returned JSX branch, or narrow the rule
configuration to the component surface that needs coverage hooks.

## Limits

A string or expression-free template key, optional chaining, one `const` alias, and `.call` or a literal-array `.apply` are the same static call when this file can see the binding. Every declarator in one `const` or `export const` statement is collected. `export const Later = Early; function Early() { return <button />; }` is still `Early`. `React.memo.apply(React, [() => <button />])` is the wrapped callback. A spread argument list is not. A computed identifier key is not a static name.

Another file's re-export is not visible here.

```tsx
// button.tsx
export function Button() { return <button>Save</button>; }
```

```tsx
// index.tsx
// This file has no local component implementation for the rule to inspect.
export { Button } from "./button";
```

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/playwright-require-exported-component-attribute -- presentational primitive intentionally exposes no test hook
export function Stack() { return <div />; }
```

## Related rules

- [`playwright-require-interactive-test-id`](playwright-require-interactive-test-id.md)
  targets interactive JSX specifically.
