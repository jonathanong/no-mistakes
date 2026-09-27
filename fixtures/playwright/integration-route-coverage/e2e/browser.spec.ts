import { test } from '@playwright/test'
test('interactive route', async ({ page }) => { await page.goto('/interactive') })
