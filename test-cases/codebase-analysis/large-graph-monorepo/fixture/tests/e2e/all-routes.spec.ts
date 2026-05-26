import { test } from '@playwright/test';
test('visits generated routes', async ({ page }) => {
  await page.goto('/area-0/item/0');
  await page.goto('/area-1/item/1');
  await page.goto('/area-2/item/0');
  await page.goto('/area-3/item/1');
  await page.goto('/area-4/item/0');
  await page.goto('/area-5/item/1');
  await page.goto('/area-6/item/0');
  await page.goto('/area-7/item/1');
  await page.goto('/area-0/item/0');
  await page.goto('/area-1/item/1');
  await page.goto('/area-2/item/0');
  await page.goto('/area-3/item/1');
  await page.goto('/area-4/item/0');
  await page.goto('/area-5/item/1');
  await page.goto('/area-6/item/0');
  await page.goto('/area-7/item/1');
});
