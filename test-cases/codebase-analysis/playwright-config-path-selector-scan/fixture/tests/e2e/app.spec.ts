import { test } from "@playwright/test";

test.describe("App", () => {
  test("saves", async ({ page }) => {
    await page.goto("/");
    await page.getByTestId("save").click();
  });
});
