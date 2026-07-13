import { test } from "@playwright/test";

test("loads the forbidden route", async ({ page }) => {
  await page.goto("/");
});
