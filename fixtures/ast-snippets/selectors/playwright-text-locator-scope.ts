test.beforeEach(async ({ page }) => {
  await page.getByText("Setup text").click();
});

test.beforeAll(async ({ page }) => {
  await page.getByText("Suite setup text").click();
});

test("uses setup", async ({ page }) => {
  await page.getByText("Test text").click();
});
