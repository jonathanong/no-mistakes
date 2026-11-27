# `no-mistakes/playwright-consistent-attribute`

## Why

One canonical test-id attribute keeps JSX hooks and Playwright selectors easy
to search, configure, and connect in static coverage analysis.

## Disallowed

```tsx
<>
  <button data-testid="save">Save</button>
  <button {...{ "data-testid": "save" }}>Save</button>
  <button ns:data-testid="save">Save</button>
</>
```

A spread whose keys are visible and not selector attributes stays quiet. A
spread the rule cannot read, and a namespaced name whose local name is a
configured selector attribute, are `unanalyzable`.

## Allowed

```tsx
<>
  <button data-pw="save">Save</button>
  <button {...{ className: "save" }}>Save</button>
</>
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
// no-mistakes-disable-next-line playwright-consistent-attribute -- vendor component contract requires data-testid
// no-mistakes-disable-line playwright-consistent-attribute -- vendor component contract requires data-testid
// no-mistakes-disable-file playwright-consistent-attribute -- vendor component contract requires data-testid
const vendorButton = <button data-testid="vendor-save">Save</button>;
```

`no-mistakes-disable-line`, `no-mistakes-disable-next-line`, and
`no-mistakes-disable-file` also suppress this rule when the comment names it.
Directive-looking string literals do not.

## Related rules

- [`playwright-literals`](playwright-literals.md) requires static test-id
  values.
- [`playwright-prefer-get-by-test-id`](playwright-prefer-get-by-test-id.md)
  uses the same attribute list in tests.
