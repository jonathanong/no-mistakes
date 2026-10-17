import { expect, test } from "@playwright/test";

test("saves", async ({ page }) => {
  await page.goto("/");
  await page.getByTestId("save").click();
  await expect(page.getByText("Save")).toBeVisible();
});
