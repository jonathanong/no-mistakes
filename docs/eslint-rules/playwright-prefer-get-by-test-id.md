# `no-mistakes/playwright-prefer-get-by-test-id`

## Why

`getByTestId` is clearer and less brittle than a CSS selector that manually
spells a configured test-id attribute.

## Disallowed

```ts
page.locator('[data-pw="save-button"]');
```

## Allowed

```ts
page.getByTestId("save-button");
```

## Options

- `selectorAttributes` lists attributes recognized in exact CSS selectors; it
  defaults to `["data-testid", "data-pw"]`.

## Fix

Replace an exact configured test-id CSS selector with `getByTestId(value)`.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/playwright-prefer-get-by-test-id -- browser-only selector syntax is under test
page.locator('[data-pw="save-button"]');
```

## Related rules

- [`playwright-selector-priority`](playwright-selector-priority.md) prefers
  semantic locators before raw selectors.
