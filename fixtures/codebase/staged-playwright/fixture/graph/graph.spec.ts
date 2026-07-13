import { test } from "@playwright/test";

test("eligible locator", async ({ page }) => {
  await page.getByRole("button", { name: "Save" }).click();
});
