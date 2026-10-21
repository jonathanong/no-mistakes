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
function SaveButton({ testId = "save-button" }) {
  return (
    <>
      <button data-pw="save-button">Save</button>
      <button data-pw={testId}>Save</button>
    </>
  );
}
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

## Limits

A string or expression-free template key, optional chaining, one `const` alias, and `.call` or a literal-array `.apply` are the same static call when this file can see the binding. A computed identifier key is not a static name.

Another file's re-export is not visible here.

```js
export { page as screen } from "./playwright.mjs";
screen.locator("#save");
```

## Suppression

```tsx
// eslint-disable-next-line no-mistakes/playwright-literals -- generated rows require runtime ids
return <button data-pw={`row-${row.id}`}>Open</button>;
```

## Related rules

- [`playwright-defaults`](playwright-defaults.md) validates the allowed prop
  default form.
- [`playwright-unique`](playwright-unique.md) catches duplicate literal IDs.
