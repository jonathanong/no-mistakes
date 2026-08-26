# `no-mistakes/playwright-literals`

## Why

Literal selector values make source-to-test coverage edges deterministic.
Computed IDs cannot be reliably matched to Playwright assertions.

## Disallowed

```tsx
<button data-pw={`save-${user.id}`}>Save</button>
```

## Allowed

```tsx
<button data-pw="save-button">Save</button>
<button data-pw={testId}>Save</button>; // when testId has a literal default
```

## Options

- `selectorAttributes` lists attributes treated as test IDs; it defaults to
  `["data-testid", "data-pw"]`.
- `allowDefaultedProps` permits a prop with a literal default; it defaults to
  `true`.
- `allowStaticTemplates` permits expression-free template literals; it defaults
  to `false`.

## Fix

Use a literal selector, or expose a prop with a literal default when a component
needs a controlled override.

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/playwright-literals -- generated rows require runtime ids
return <button data-pw={`row-${row.id}`}>Open</button>;
```

## Related rules

- [`playwright-defaults`](playwright-defaults.md) validates the allowed prop
  default form.
- [`playwright-unique`](playwright-unique.md) catches duplicate literal IDs.
