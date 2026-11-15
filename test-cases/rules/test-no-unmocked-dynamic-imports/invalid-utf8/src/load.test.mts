import { test } from 'vitest'

test('loads', async () => {
  await import('./value.mts')
})
