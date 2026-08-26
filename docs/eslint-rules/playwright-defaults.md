# `no-mistakes/playwright-defaults`

## Why

A component that passes a test-id prop into JSX needs a literal default so
static analysis can identify the selector when the caller omits the prop.

## Disallowed

```tsx
export function SaveButton({ testId }: { testId?: string }) {
  return <button data-pw={testId}>Save</button>;
}
```

## Allowed

```tsx
export function SaveButton({ testId = "save-button" }: { testId?: string }) {
  return <button data-pw={testId}>Save</button>;
}
```

## Options

- `selectorAttributes` lists attributes treated as test IDs. It defaults to
  `["data-testid", "data-pw"]`.

## Fix

Give the passed-through prop a literal default, or use a literal test-id value
at the JSX site.

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/playwright-defaults -- selector is supplied by a generated host
return <button data-pw={testId}>Save</button>;
```

## Related rules

- [`playwright-literals`](playwright-literals.md) controls which dynamic forms
  remain acceptable.
