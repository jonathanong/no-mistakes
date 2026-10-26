import { test } from '@playwright/test';

test('visits the only configured route', async ({ page }) => {
  await page.goto('/');
});
