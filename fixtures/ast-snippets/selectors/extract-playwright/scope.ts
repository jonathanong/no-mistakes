await page.getByTestId("file-scope");
test.beforeEach(async ({ page }) => {
  await page.getByTestId("setup");
});
test.afterEach(async ({ page }) => {
  await page.getByTestId("teardown");
});
test(`dynamic ${name}`, async ({ page }) => {
  await page.getByTestId("dynamic-test");
});
test("active", async ({ page }) => {
  await page.getByTestId("inside-test");
});
