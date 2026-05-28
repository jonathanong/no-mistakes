import { test, expect } from "@playwright/test";

test.describe("User profile", () => {
  test("loads a specific user profile", async ({ page }) => {
    await page.goto("/users/123");
    await expect(page).toHaveURL(/\/users\//);
  });
});
