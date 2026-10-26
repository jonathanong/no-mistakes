import { test } from 'vitest'

test('conditional setup helper', async () => {
  await import('./conditional-target.mts') // EXPECT_UNMOCKED
})
