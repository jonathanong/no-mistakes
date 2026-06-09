import { test } from '@playwright/test';

test('navigates home and clicks save by test id', async ({ page }) => {
  await page.goto('/');
  await page.getByTestId('save').click();
});
