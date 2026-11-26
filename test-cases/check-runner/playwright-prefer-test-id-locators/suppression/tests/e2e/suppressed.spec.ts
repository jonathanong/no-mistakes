// no-mistakes-disable-file playwright-prefer-test-id-locators
import { test } from '@playwright/test';

test('save widget', async ({ page }) => {
  await page.goto('/');
  await page.getByText('Save').click();
});
