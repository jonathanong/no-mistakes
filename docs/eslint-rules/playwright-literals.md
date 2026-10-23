# `no-mistakes/playwright-literals`

## Why

Literal selector values make source-to-test coverage edges deterministic.
Computed IDs cannot be reliably matched to Playwright assertions.

## Disallowed

```tsx
<button data-pw={`save-${user.id}`}>Save</button>
```

```tsx
let find = page.getByTestId;
find(`save-${user.id}`);
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

```tsx
let find = page.getByTestId;
find = page.locator;
find(`save-${user.id}`);
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

A string or expression-free template key, optional chaining, one single-write `const`, `let`, or `var` alias, and `.call` or a literal-array `.apply` are the same static call when this file can see the binding. A computed identifier key is not a static name.

Another file's re-export is not visible here.

```js
// selectors.mjs (page-methods.mjs exports a bound getByTestId helper)
export { getByTestId as find } from "./page-methods.mjs";
```

```js
// e2e/save.spec.js
import { find } from "../selectors.mjs";
// The imported alias is not known to be getByTestId in this file.
find(`save-${user.id}`);
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
