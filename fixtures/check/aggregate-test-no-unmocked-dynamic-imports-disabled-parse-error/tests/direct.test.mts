import { test } from 'vitest'

test('unmocked import remains reportable', async () => {
  await import('../src/leaf.mts')
})
