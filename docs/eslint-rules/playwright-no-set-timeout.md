# `no-mistakes/playwright-no-set-timeout`

## Why

Fixed sleeps make tests slow and flaky because elapsed time is not proof that
the UI reached the expected state.

## Disallowed

```ts
await page.waitForTimeout(500);
await new Promise((resolve) => setTimeout(resolve, 500));
```

## Allowed

```ts
await expect(page.getByRole("status")).toHaveText("Saved");
await page.waitForURL("**/complete");
```

## Options

This rule has no options.

## Fix

Wait for a route, locator state, response, or assertion that represents the
actual completion condition.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/playwright-no-set-timeout -- intentionally measures a timer-based retry UI
await page.waitForTimeout(100);
```

## Related rules

- [`playwright-assertion-timeout-cap`](playwright-assertion-timeout-cap.md)
  caps long assertion waits.
