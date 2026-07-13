import { test } from "@playwright/test";

test("uses the malformed component selector", async ({ page }) => {
  await page.getByTestId("save").click();
});
