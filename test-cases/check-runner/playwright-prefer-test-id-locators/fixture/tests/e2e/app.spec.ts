import { test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.goto("/");
});

test("flags copy-coupled locators with test id alternatives", async ({ page }) => {
  await page.getByRole("button", { name: "Save" }).click();
  await page.getByLabel("Email").fill("reader@example.com");
  await page.getByPlaceholder("Search").fill("topic");
  await page.getByAltText("Company logo").click();
  await page.getByTitle("Help").click();
  await page.getByText("Untracked copy").click();
  await page.getByText(label).click();
});

test("suppresses intentional copy locator", async ({ page }) => {
  // no-mistakes-disable-next-line playwright-prefer-test-id-locators: accessibility assertion
  await page.getByText("Save").click();
});
