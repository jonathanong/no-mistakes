import { test, expect } from "@playwright/test";

test.describe("Dashboard", () => {
  test("loads the dashboard page", async ({ page }) => {
    await page.goto("/dashboard");
    await expect(page).toHaveURL(/\/dashboard/);
  });
});
