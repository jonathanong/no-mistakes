import { test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.goto("/");
});

test("covers visible button text", async ({ page }) => {
  await page.getByRole("button", { name: "Discuss" }).click();
  await page.getByLabel("Email").fill("reader@example.com");
  await page.getByPlaceholder("Search").fill("topic");
  await page.getByText("Send").click();
});

test("covers visible text through adjacent selector", async ({ page }) => {
  await page.locator('[data-pw="discuss-in-community-button"]').click();
  await page.getByText("Discuss", { exact: true }).click();
});

test("uses setup route for text locator coverage", async ({ page }) => {
  await page.getByRole("button", { name: "save" }).click();
});

test.describe("nested suite", () => {
  test("uses parent setup route for nested text locator coverage", async ({ page }) => {
    await page.getByRole("button", { name: "request" }).click();
  });
});

test.describe("late setup", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
  });

  test("uses later setup route for text locator coverage", async ({ page }) => {
    await page.getByText("Send").click();
  });
});

test.skip("skipped text locator stays policy gated", async ({ page }) => {
  await page.getByText("Discuss").click();
});
