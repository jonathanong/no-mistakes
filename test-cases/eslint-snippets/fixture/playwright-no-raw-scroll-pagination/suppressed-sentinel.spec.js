import { test } from "@playwright/test";

test("checks the sentinel itself", async ({ page }) => {
  const sentinel = page.getByTestId("load-more");
  // eslint-disable-next-line no-mistakes/playwright-no-raw-scroll-pagination -- inspect the sentinel itself
  await sentinel.scrollIntoViewIfNeeded();
  // no-mistakes-disable-next-line playwright-no-raw-scroll-pagination -- inspect the sentinel itself
  await page.getByTestId("load-more").scrollIntoViewIfNeeded();
});
