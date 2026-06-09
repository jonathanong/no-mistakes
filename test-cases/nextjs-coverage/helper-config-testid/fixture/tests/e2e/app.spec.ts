import { test } from '@playwright/test';

test('covers the save button via getByTestId', async ({ page }) => {
  await page.goto('/');
  await page.getByTestId('save').click();
});
