# `no-mistakes/playwright-assertion-timeout-cap`

## Why

Large assertion-specific timeouts hide slow or flaky UI conditions. A bounded
timeout keeps tests focused on observable readiness.

## Disallowed

```ts
// e2e/status.spec.ts
await expect(page.getByRole("status")).toHaveText("Done", { timeout: 30_000 });
```

```ts
// e2e/status.spec.ts
import { expect as check } from "@playwright/test";

let verify = check;
await verify(page.getByRole("status")).toHaveText("Done", { timeout: 30_000 });
```

```ts
// e2e/namespace.spec.ts
import * as playwright from "@playwright/test";

let pw = playwright;
await pw.expect(page.getByRole("status")).toBeVisible({ timeout: 30_000 });
```

```ts
// e2e/shorthand.spec.ts
import * as playwright from "@playwright/test";

let { expect } = playwright;
await expect(page.getByRole("status")).toBeVisible({ timeout: 30_000 });
```

## Allowed

```ts
// e2e/status.spec.ts
await expect(page.getByRole("status")).toHaveText("Done", { timeout: 5_000 });
```

```ts
// e2e/status.spec.ts
import { expect as check } from "@playwright/test";

let verify = check;
verify = custom;
await verify(page.getByRole("status")).toHaveText("Done", { timeout: 30_000 });
```

## Options

- `max` is the largest permitted assertion timeout in milliseconds. It defaults
  to `10000`.

## Fix

Wait for a specific UI condition, improve the setup, or adjust the test's
overall budget rather than increasing one assertion beyond the cap.

## Limits

A string or expression-free template key, optional chaining, one single-write `const`, `let`, or `var` alias, and `.call` or a literal-array `.apply` are the same static call when this file can see the binding. `pw.expect` counts only when `pw` is a namespace import from `@playwright/test` or a single-write alias of that namespace. A single-write `{ expect }` destructure of that namespace counts too. `const check = customAssertions.expect` is not that namespace. `const verify = check` still counts when `check` is a renamed `expect` import. A computed identifier key is not a static name.

Another file's re-export is not visible here.

```js
// assertions.mjs
export { expect as verify } from "@playwright/test";
```

```js
// e2e/status.spec.js
import { verify } from "../assertions.mjs";
// The imported alias is not known to be Playwright expect in this file.
verify(page.getByRole("status")).toBeVisible({ timeout: 30_000 });
```

## Suppression

```ts
// eslint-disable-next-line no-mistakes/playwright-assertion-timeout-cap -- third-party report has a documented 20s SLA
await expect(report).toBeVisible({ timeout: 20_000 });
```

## Related rules

This rule owns assertion-specific timeout caps.
