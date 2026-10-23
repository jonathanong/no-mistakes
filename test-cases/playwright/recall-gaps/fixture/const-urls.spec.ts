import { test } from '@playwright/test'
import { imported } from './imported'

const slug = 'outer'
const section = `\x70osts`

test('const template URLs', async ({ page }) => {
  const slug = 'post'
  await page.goto(`/blog/${slug}`)
  await page.waitForURL(`/archive/${section}/${slug}`)
  const escaped = 'a\u0062'
  await page.goto(`/escaped/${escaped}`)
  {
    const slug = `inner`
    await page.goto(`/nested/${slug}`)
  }
  await page.goto(`/after/${slug}`)
})

test('unresolved bindings stay dynamic', async ({ page, slug }) => {
  await page.goto(`/parameter/${slug}`)
  let mutable = 'mutable'
  var legacy = 'legacy'
  const number = 42
  const dynamic = `${slug}`
  await page.goto(`/let/${mutable}`)
  await page.goto(`/var/${legacy}`)
  await page.goto(`/number/${number}`)
  await page.goto(`/dynamic/${dynamic}`)
  await page.goto(`/import/${imported}`)
  // A later block binding shadows the outer constant even before initialization.
  {
    await page.goto(`/before/${slug}`)
    const slug = 'late'
  }
})

test('outer constant', async ({ page }) => {
  await page.goto(`/outer/${slug}`)
})

// Invalid writes and non-literal aliases must never masquerade as static paths.
test('unsafe constants', async ({ page }) => {
  const written = 'old'
  written = 'new'
  const object = { slug: 'value' }
  const alias = slug
  await page.goto(`/written/${written}`)
  await page.goto(`/object/${object}`)
  await page.goto(`/alias/${alias}`)
  await page.goto(`/expression/${slug.toUpperCase()}`)
  await page.click(`a[href="/click/${slug}"]`)
})
