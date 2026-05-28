import { test, expect } from "@playwright/test";

test.describe("Users API", () => {
  test("returns the users list", async ({ page, request }) => {
    const response = await request.get("/api/users");
    expect(response.ok()).toBeTruthy();
    await page.goto("/users");
  });
});
