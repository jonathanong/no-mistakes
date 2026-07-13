test('visits the app route', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Save' }).click();
});
