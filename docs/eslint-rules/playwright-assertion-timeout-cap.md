# `no-mistakes/playwright-assertion-timeout-cap`

## Why

Large assertion-specific timeouts hide slow or flaky UI conditions. A bounded
timeout keeps tests focused on observable readiness.

## Disallowed

```ts
// e2e/status.spec.ts
await expect(page.getByRole("status")).toHaveText("Done", { timeout: 30_000 });
```

## Allowed

```ts
// e2e/status.spec.ts
await expect(page.getByRole("status")).toHaveText("Done", { timeout: 5_000 });
```

## Options

- `max` is the largest permitted assertion timeout in milliseconds. It defaults
  to `10000`.

## Fix

Wait for a specific UI condition, improve the setup, or adjust the test's
overall budget rather than increasing one assertion beyond the cap.

## Limits

A string or expression-free template key, optional chaining, one `const` alias, and `.call` or a literal-array `.apply` are the same static call when this file can see the binding. A computed identifier key is not a static name.

Another file's re-export is not visible here.

```js
export { page as screen } from "./playwright.mjs";
screen.locator("#save");
```

## Suppression

```ts
// eslint-disable-next-line no-mistakes/playwright-assertion-timeout-cap -- third-party report has a documented 20s SLA
await expect(report).toBeVisible({ timeout: 20_000 });
```

## Related rules

This rule owns assertion-specific timeout caps.
