import { test } from '@playwright/test';

test('home', async ({ page }) => {
  await page.goto('/');
  await page.locator('#literal-unique').isVisible();
});
