# `no-mistakes/playwright-no-empty`

## Why

An empty test ID is neither a stable selector nor meaningful coverage evidence.

## Disallowed

```tsx
<button data-pw="">Save</button>
```

## Allowed

```tsx
<button data-pw="save-button">Save</button>
```

## Options

- `selectorAttributes` lists test-id attributes; it defaults to
  `["data-testid", "data-pw"]`.

## Fix

Use a descriptive non-empty literal or remove the test-id attribute when the
element should not be a selector target.

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/playwright-no-empty -- fixture verifies legacy empty-attribute rendering
return <button data-pw="">Save</button>;
```

## Related rules

- [`playwright-naming-convention`](playwright-naming-convention.md) validates
  the non-empty value's naming style.
