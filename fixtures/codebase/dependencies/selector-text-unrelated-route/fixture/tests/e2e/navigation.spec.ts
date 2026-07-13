import { test } from "@playwright/test";

test("navigates in an unrelated test file", async ({ page }) => {
  await page.goto("/");
});
