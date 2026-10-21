# `no-mistakes/playwright-require-interactive-test-id`

## Why

Interactive controls are high-value test targets. A stable test ID makes their
coverage and automation intent explicit.

## Disallowed

```tsx
<button onClick={save}>Save</button>
```

## Allowed

```tsx
<button data-pw="save-button" onClick={save}>Save</button>
```

## Options

- `selectorAttributes` lists acceptable test-id attributes; it defaults to
  `["data-testid", "data-pw"]`.
- `interactiveComponents` adds component names considered interactive. Entries
  may be exact names or `/regex/` patterns.

## Fix

Add a configured literal test ID to the interactive element, or configure a
project-specific interactive component name.

## Limits

A string or expression-free template key, optional chaining, one `const` alias, and `.call` or a literal-array `.apply` are the same static call when this file can see the binding. A nested component that reuses an imported local name is that local component, not the import. `const Alias = Save` is the import only when that `Save` binding is the import, so `function Wrapper(Save) { const Alias = Save; return <Alias />; }` is not `SaveButton`. A computed identifier key is not a static name.

Another file's re-export is not visible here.

```js
export { page as screen } from "./playwright.mjs";
screen.locator("#save");
```

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/playwright-require-interactive-test-id -- decorative control is deliberately not automation-facing
const decorative = <button onClick={animate}>Play</button>;
```

## Related rules

- [`playwright-require-exported-component-attribute`](playwright-require-exported-component-attribute.md)
  covers exported component roots and branches.
