import { test } from "@playwright/test";

test("control saves", async ({ page }) => {
  await page.goto("/control");
  await page.getByTestId("control-save").click();
});
