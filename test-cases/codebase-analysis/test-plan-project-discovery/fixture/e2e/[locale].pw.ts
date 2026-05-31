import { test } from '@playwright/test'

test('localized home page', async ({ page }) => {
  await page.goto('/en')
})
