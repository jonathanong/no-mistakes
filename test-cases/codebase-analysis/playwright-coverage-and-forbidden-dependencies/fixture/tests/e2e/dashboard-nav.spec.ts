import { test, expect } from '@playwright/test'

test.describe('DashboardNav', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/account/settings')
  })

  test('renders the nav', async ({ page }) => {
    await expect(page.getByTestId('dashboard-nav')).toBeVisible()
  })

  test('account tab is active', async ({ page }) => {
    await expect(page.getByTestId('nav-tab-account')).toBeVisible()
  })
})
