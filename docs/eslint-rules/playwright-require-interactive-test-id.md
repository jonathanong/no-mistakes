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

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/playwright-require-interactive-test-id -- decorative control is deliberately not automation-facing
const decorative = <button onClick={animate}>Play</button>;
```

## Related rules

- [`playwright-require-exported-component-attribute`](playwright-require-exported-component-attribute.md)
  covers exported component roots and branches.
