import { expect } from "@playwright/test";

declare const loc: unknown;

expect(loc).toBeVisible({ timeout: 15000 });
expect(loc).toBeVisible({ timeout: 15000 } as const);
expect(loc).toBeVisible({ timeout: 15000 as number });
expect(loc).toBeVisible({ ["timeout" as const]: 15000 });
expect(loc)!.toBeVisible({ timeout: 15000 });
(expect(loc).toBeVisible as any)({ timeout: 15000 });
expect(loc).toBeVisible({ timeout: 1000 });
