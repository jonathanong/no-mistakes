import { test } from 'vitest'

test('doMock setup coverage', async () => {
  await import('./do-target.mts') // EXPECT_UNMOCKED
})
