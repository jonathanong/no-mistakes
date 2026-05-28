import { test, expect, Page } from "@playwright/test";

async function navigateTo(page: Page, url: string) {
  await page.goto(url);
}

test("news page renders via navigateTo helper", async ({ page }) => {
  await navigateTo(page, "/news");
  await expect(page.getByTestId("news-item-card")).toBeVisible();
});
