import { test } from "@playwright/test";

test("covers a self-closing selector", async ({ page }) => {
  await page.getByTestId("save").click();
});

// These locators must not demand app-text/reachability work: one is skipped
// and the other runs only during teardown.
test.skip("ignores skipped visible text", async ({ page }) => {
  await page.getByText("Unrelated visible app text").click();
});

test.afterEach(async ({ page }) => {
  await page.getByText("Unrelated visible app text").click();
});
