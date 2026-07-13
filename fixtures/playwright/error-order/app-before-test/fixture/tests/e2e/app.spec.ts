import { test } from "@playwright/test";

test("malformed test", async ({ page }) => {
  await page.getByTestId("bad").click(
});
