import { test } from "@playwright/test";
test("x", async ({ page }) => { await page.locator('[data-pw="x"]').click(); });
