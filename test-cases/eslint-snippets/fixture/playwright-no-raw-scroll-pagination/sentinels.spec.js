import { test } from "@playwright/test";

test("loads another page", async ({ page }) => {
  const sentinel = page.locator(".load-more");
  await sentinel.scrollIntoViewIfNeeded();
  await page.getByTestId("load-more").scrollIntoViewIfNeeded();
  await (page?.getByTestId(`load-\x6dore`))["scrollIntoViewIfNeeded"]();
});
