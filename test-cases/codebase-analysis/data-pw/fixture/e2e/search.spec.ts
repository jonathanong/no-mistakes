import { test, expect } from "@playwright/test";

test("search", async ({ page }) => {
  // CSS attribute selector form is matched.
  await page.locator('[data-pw="search-bar"]').click();
  // getByTestId form is intentionally NOT matched (documented limit).
  await page.getByTestId("search-bar").fill("hi");
  await expect(page).toHaveTitle(/Search/);
});
