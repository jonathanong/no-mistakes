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

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/playwright-require-exported-component-attribute -- presentational primitive intentionally exposes no test hook
export function Stack() { return <div />; }
```

## Related rules

- [`playwright-require-interactive-test-id`](playwright-require-interactive-test-id.md)
  targets interactive JSX specifically.
