import { test } from "@playwright/test";

test("uses the forbidden selector", async ({ page }) => {
  await page.getByTestId("danger").click();
});
