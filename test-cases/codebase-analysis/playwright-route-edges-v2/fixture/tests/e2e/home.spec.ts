import { test } from "@playwright/test";

test("loads home", async ({ page }) => {
  await page.goto("/landing");
});
