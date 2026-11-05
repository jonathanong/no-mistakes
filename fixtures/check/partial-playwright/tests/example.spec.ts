import { test, expect } from '@playwright/test';
test('save', async ({ page }) => { await page.goto('/'); await expect(page.getByTestId('save')).toBeVisible(); });
