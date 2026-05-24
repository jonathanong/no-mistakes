test.beforeEach(async ({ page }) => {
  await page.getByText("Setup text").click();
});

test("uses setup", async ({ page }) => {
  await page.getByText("Test text").click();
});
