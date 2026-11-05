import { test } from '@playwright/test';
test('good', async ({ page }) => { await page.getByText('Hello').click(); });
