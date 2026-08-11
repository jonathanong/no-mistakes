import { test } from 'vitest'

test('visible test remains in the bounded request inventory', async () => {
  await import('../src/leaf.mts')
})
