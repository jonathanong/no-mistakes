import { test } from "@playwright/test";

await expect(locator).toBeVisible({ timeout: 20000 });
await page.locator(".save").click();
await page.locator("h2").click();
await page.locator("text=Save").click();
setTimeout(() => {}, 100);
await page.waitForTimeout(100);
await page["waitForTimeout"](100);
