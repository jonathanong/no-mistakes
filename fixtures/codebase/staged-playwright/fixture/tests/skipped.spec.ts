import { test } from "@playwright/test";

test.skip("skipped locator", async ({ page }) => {
  await page.getByText("Skipped text").click();
});

test.afterAll(async ({ page }) => {
  await page.getByText("Teardown text").click();
});
