import { test } from "@playwright/test";

// Control case: a `const` interpolated into a direct `page.goto` template literal. Like the
// `let` cases this stays an unresolved `${...}` wildcard and must edge the dynamic
// `[idOrUsername]` page, never the literal `/user/settings` page.
const TEST_USER_USERNAME = "seeded-user";

test("goto template with const", async ({ page }) => {
  await page.goto(`/user/${TEST_USER_USERNAME}`);
});
