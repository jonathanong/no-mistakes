import { test } from "@playwright/test";

test("merged extraction settings", async ({ page }) => {
  await goA(page, "/a");
  await goB(page, "/b");
  await page.locator('[data-a="one"]');
  await page.locator('[data-b="two"]');
  await page.locator('[component-a="one"]');
  await page.locator('[component-b="two"]');
  await page.getByTestId("merged");
  await page.getByTestId("only-b");
});

test.skip("shared skipped text", async ({ page }) => {
  await page.getByText("shared text");
});
