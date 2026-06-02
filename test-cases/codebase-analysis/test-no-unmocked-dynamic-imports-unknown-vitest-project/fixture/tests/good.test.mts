import { test, expect } from 'vitest'

test('uses a static import path', async () => {
  const module = await import('../src/lazy.mts')
  expect(module.value).toBe(1)
})
