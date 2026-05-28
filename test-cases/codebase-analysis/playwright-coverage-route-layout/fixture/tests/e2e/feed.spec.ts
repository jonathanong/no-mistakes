import { test, expect } from "@playwright/test";

test("news page renders", async ({ page }) => {
  await page.goto("/news");
  await expect(page.getByTestId("news-item-card")).toBeVisible();
});
