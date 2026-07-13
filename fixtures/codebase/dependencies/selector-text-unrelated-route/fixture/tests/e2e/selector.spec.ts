import { test } from "@playwright/test";

test("uses an adjacent selector without navigation", async ({ page }) => {
  await page.getByTestId("discuss").click();
  await page.getByRole("button", { name: "Discuss" }).click();
});
