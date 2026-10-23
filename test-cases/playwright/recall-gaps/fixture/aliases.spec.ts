import { test } from '@playwright/test'

test('coverage aliases', async ({ page }) => {
  await page.goto('/dashboard')
  await page.getByTestId('save')

  await page['goto']('/settings')
  await page['getByTestId']('archive')
  await page[`getByTestId`]('rename')

  const id = 'publish'
  await page.getByTestId(id)

  const open = page.goto
  await open('/billing')

  const slug = 'post'
  await page.goto(`/blog/${slug}`)
})
