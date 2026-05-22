test("user route", async ({ page }) => {
  await page.goto("/users/42");
});
