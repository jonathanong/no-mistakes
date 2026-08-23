import { test } from "@playwright/test";

test("selector", async ({ page }) => {
  await page.getByTestId("old-selector").click();
});
