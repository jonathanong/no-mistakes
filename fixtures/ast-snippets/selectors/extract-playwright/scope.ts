await page.getByTestId("file-scope");
test.beforeEach(async ({ page }) => {
  await page.getByTestId("setup");
});
test("active", async ({ page }) => {
  await page.getByTestId("inside-test");
});
