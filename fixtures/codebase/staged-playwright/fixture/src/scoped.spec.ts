import { test } from "@playwright/test";

test("direct selector only", async ({ page }) => {
  await page.getByTestId("scoped").click();
});
