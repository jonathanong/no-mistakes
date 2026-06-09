import { test } from "@playwright/test";

// Regression for #391: navigation paths whose interpolation is unresolvable at analysis
// time must still emit a [route] edge to the dynamic `[idOrUsername]` page, and must NOT
// edge to the sibling literal `/user/settings` page.
const TEST_USER_USERNAME = "seeded-user";

test.describe("user pages", () => {
  // `let` reassigned from a runtime call in real suites; unresolvable at analysis time.
  let targetUsername = "";

  test("template literal nav with unresolved let", async ({ page }) => {
    await navigateTo(page, `/user/${targetUsername}`);
  });

  test("string concatenation nav", async ({ page }) => {
    await navigateTo(page, "/user/" + targetUsername);
  });

  test("goto template with const", async ({ page }) => {
    await page.goto(`/user/${TEST_USER_USERNAME}`);
  });
});
