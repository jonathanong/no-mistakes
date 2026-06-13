import { test, type Page } from '@playwright/test';

function getAsideLocator(page: Page, dataPw: string) {
  return page.getByTestId(dataPw).first();
}

test('covers route and uses helper wrapper', async ({ page }) => {
  await page.goto('/');
  await getAsideLocator(page, 'example-button').click();
});
