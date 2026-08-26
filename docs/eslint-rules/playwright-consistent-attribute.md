# `no-mistakes/playwright-consistent-attribute`

## Why

One canonical test-id attribute keeps JSX hooks and Playwright selectors easy
to search, configure, and connect in static coverage analysis.

## Disallowed

```tsx
<button data-testid="save">Save</button>
```

## Allowed

```tsx
<button data-pw="save">Save</button>
```

## Options

- `selectorAttributes` lists recognized test-id attributes; it defaults to
  `["data-testid", "data-pw"]`.
- `canonicalAttribute` is the only accepted attribute among that set; it
  defaults to `"data-pw"`.

## Fix

Rename recognized non-canonical attributes to the configured canonical name.

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/playwright-consistent-attribute -- vendor component contract requires data-testid
const vendorButton = <button data-testid="vendor-save">Save</button>;
```

## Related rules

- [`playwright-literals`](playwright-literals.md) requires static test-id
  values.
- [`playwright-prefer-get-by-test-id`](playwright-prefer-get-by-test-id.md)
  uses the same attribute list in tests.
