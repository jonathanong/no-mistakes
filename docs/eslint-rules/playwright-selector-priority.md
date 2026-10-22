# `no-mistakes/playwright-selector-priority`

## Why

Semantic locators describe user-visible intent and survive markup changes better
than raw CSS, tag, or `text=` selectors.

## Disallowed

```ts
// e2e/save.spec.ts
page.locator("button.submit");
page.locator("text=Save");
page.locator("h2");
```

## Allowed

```ts
// e2e/save.spec.ts
page.getByRole("button", { name: "Save" });
page.getByText("Save");
page.getByRole("heading", { level: 2 });
```

## Options

This rule has no options.

## Fix

Use `getByRole`, `getByLabel`, or `getByText` when the UI has semantic text;
use a test ID only when a semantic locator is not appropriate.

## Limits

A string or expression-free template key, optional chaining, one `const` alias, and `.call` or a literal-array `.apply` are the same static call when this file can see the binding. A computed identifier key is not a static name. A parameter named `locator`, including a `const` alias of that parameter, does not establish a Playwright method binding.

Another file's re-export is not visible here.

```js
export { page as screen } from "./playwright.mjs";
screen.locator("#save");
```

## Suppression

```ts
// eslint-disable-next-line no-mistakes/playwright-selector-priority -- verifies generated CSS selector contract
page.locator(".generated-widget");
```

## Related rules

- [`playwright-prefer-get-by-test-id`](playwright-prefer-get-by-test-id.md)
  gives the preferred form for exact test-id CSS selectors.
