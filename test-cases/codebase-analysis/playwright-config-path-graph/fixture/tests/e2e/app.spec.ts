import { test } from "@playwright/test";

test("saves", async ({ page }) => {
  await page.goto("/");
  await page.getByTestId("save").click();
});
