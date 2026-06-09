import { test, expect } from '@playwright/test';

test('chat requires credentials', async ({ page }) => {
  await page.goto('/chat');
  await expect(page.getByTestId('chat')).toBeVisible();
});
