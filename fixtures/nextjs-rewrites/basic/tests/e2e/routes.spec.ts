import { test } from "@playwright/test";

test("posts page", async ({ page }) => {
  await page.goto("/posts/hello");
});

test("reviews page", async ({ page }) => {
  await page.goto("/reviews/best-product");
});
