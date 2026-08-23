import { test } from "@playwright/test";

test("agent saves", async ({ page }) => {
  await page.goto("/agent");
  await page.getByTestId("agent-save").click();
});
