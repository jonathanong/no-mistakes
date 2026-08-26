# `no-mistakes/playwright-assertion-timeout-cap`

## Why

Large assertion-specific timeouts hide slow or flaky UI conditions. A bounded
timeout keeps tests focused on observable readiness.

## Disallowed

```ts
await expect(page.getByRole("status")).toHaveText("Done", { timeout: 30_000 });
```

## Allowed

```ts
await expect(page.getByRole("status")).toHaveText("Done", { timeout: 5_000 });
```

## Options

- `max` is the largest permitted assertion timeout in milliseconds. It defaults
  to `10000`.

## Fix

Wait for a specific UI condition, improve the setup, or adjust the test's
overall budget rather than increasing one assertion beyond the cap.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/playwright-assertion-timeout-cap -- third-party report has a documented 20s SLA
await expect(report).toBeVisible({ timeout: 20_000 });
```

## Related rules

- [`playwright-no-set-timeout`](playwright-no-set-timeout.md) rejects fixed
  sleeps in favor of observable conditions.
