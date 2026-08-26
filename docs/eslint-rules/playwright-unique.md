# `no-mistakes/playwright-unique`

## Why

Duplicate literal test IDs within one file make selectors ambiguous and weaken
coverage evidence.

## Disallowed

```tsx
<><button data-pw="save">Save</button><button data-pw="save">Save all</button></>;
```

## Allowed

```tsx
<><button data-pw="save">Save</button><button data-pw="save-all">Save all</button></>;
```

## Options

- `selectorAttributes` lists test-id attributes; it defaults to
  `["data-testid", "data-pw"]`.

## Fix

Rename one literal ID to describe its distinct control. Dynamic values are not
treated as a proof of uniqueness.

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/playwright-unique -- fixture intentionally demonstrates duplicate markup
const fixture = <><i data-pw="item" /><i data-pw="item" /></>;
```

## Related rules

- [`playwright-naming-convention`](playwright-naming-convention.md) keeps the
  resulting identifiers consistent.
