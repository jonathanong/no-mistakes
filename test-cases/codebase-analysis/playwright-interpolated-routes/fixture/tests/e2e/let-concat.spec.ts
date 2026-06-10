import { test } from "@playwright/test";

// #391/#397: string-concatenation navigation with an unresolved `let` tail. `"/user/" +
// targetUsername` folds to `/user/${targetUsername}`, which must edge the dynamic
// `[idOrUsername]` page and must NOT edge the literal `/user/settings` page. Keep the `+`
// concatenation form — it exercises a different extraction path than the template literal.
test.describe("user pages (let concatenation)", () => {
  let targetUsername = "";

  test("string concatenation nav with unresolved let", async ({ page }) => {
    await navigateTo(page, "/user/" + targetUsername);
  });
});
