import { test } from '@playwright/test';

test('locates the used button', async ({ page }) => {
  await page.goto('/');
  await page.getByTestId('used').click();
});
