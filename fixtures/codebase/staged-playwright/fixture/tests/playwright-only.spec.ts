import { test } from "@playwright/test";

test("playwright-only selector", async ({ page }) => {
  await page.getByTestId("only").click();
});
