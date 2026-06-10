import { test } from "@playwright/test";

// #391/#397: a describe-scoped `let` initialized to "" is unresolvable at analysis time. The
// template-literal navigation must still emit a [route] edge to the dynamic `[idOrUsername]`
// page and must NOT edge the sibling literal `/user/settings` page. Do not "simplify" the empty
// initializer or the `let` away — the empty-string init is the case under test.
test.describe("user pages (let template literal)", () => {
  let targetUsername = "";

  test("template literal nav with unresolved let", async ({ page }) => {
    await navigateTo(page, `/user/${targetUsername}`);
  });
});
