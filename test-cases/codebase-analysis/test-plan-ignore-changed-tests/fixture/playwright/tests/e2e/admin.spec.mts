import { test } from "@playwright/test";

test("admin", async ({ page }) => {
  await page.goto("/admin");
});
