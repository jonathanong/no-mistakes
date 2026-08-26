# `no-mistakes/playwright-naming-convention`

## Why

A consistent test-id vocabulary makes selectors legible and reduces accidental
near-duplicates.

## Disallowed

```tsx
<button data-pw="SaveButton">Save</button>
```

## Allowed

```tsx
<button data-pw="save-button">Save</button>
```

## Options

- `selectorAttributes` lists test-id attributes; it defaults to
  `["data-testid", "data-pw"]`.
- `pattern` is the required regular-expression string. It defaults to the
  plugin's kebab-case pattern.

## Fix

Rename the literal to a value matching the configured pattern, or change the
pattern only when the repository naming policy changes.

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/playwright-naming-convention -- third-party compatibility selector
return <button data-pw="Legacy_Save">Save</button>;
```

## Related rules

- [`playwright-no-empty`](playwright-no-empty.md) rejects empty values.
- [`playwright-unique`](playwright-unique.md) rejects duplicates in a file.
