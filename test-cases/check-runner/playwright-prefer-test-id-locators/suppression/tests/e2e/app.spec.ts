import { test } from '@playwright/test';

test('save widget', async ({ page }) => {
  await page.goto('/');
  await page.getByText('Save').click(); // no-mistakes-disable-line playwright-prefer-test-id-locators
  await page.getByText('Save').click();
});
