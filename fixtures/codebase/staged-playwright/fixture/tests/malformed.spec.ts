import { test } from "@playwright/test";

test("malformed", async ({ page }) => {
  await page.getByText("broken").click(
});
