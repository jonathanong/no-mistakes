import { test } from "@playwright/test";

test("route-only consumer with text", async ({ page }) => {
  await page.goto("/route-only");
  await page.getByText("route text");
});
