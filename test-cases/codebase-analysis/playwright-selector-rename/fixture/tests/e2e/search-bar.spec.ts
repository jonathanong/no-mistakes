import { test, expect } from "@playwright/test";

test.describe("SearchBar", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/search");
  });

  test("renders the search bar", async ({ page }) => {
    await expect(page.getByTestId("search-bar")).toBeVisible();
  });
});
