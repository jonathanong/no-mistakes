import { test } from '@playwright/test';

test('story', async ({ page }) => {
  await page.goto('/');
});
