import { test } from '@playwright/test';

// Sentinel: test-side const locators are unresolved at max and low today.
const ID = 'x';

test('home', async ({ page }) => {
  await page.goto('/');
  await page.getByTestId('save').click();
  await page.getByTestId(ID).click();
});
