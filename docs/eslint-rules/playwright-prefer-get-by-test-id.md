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

## Limits

A string or expression-free template key, optional chaining, one `const` alias, and `.call` or a literal-array `.apply` are the same static call when this file can see the binding. A computed identifier key is not a static name. A line suppression on one `dragAndDrop` selector does not hide the other selector.

Another file's re-export is not visible here.

```js
export { page as screen } from "./playwright.mjs";
screen.locator("#save");
```

## Suppression

```ts
// eslint-disable-next-line no-mistakes/playwright-prefer-get-by-test-id -- browser-only selector syntax is under test
page.locator('[data-pw="save-button"]');
```

## Related rules

- [`playwright-selector-priority`](playwright-selector-priority.md) prefers
  semantic locators before raw selectors.
