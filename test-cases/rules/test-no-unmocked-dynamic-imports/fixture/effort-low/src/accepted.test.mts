import { test } from 'vitest'

test('accepted setup coverage', async () => {
  await import('./static-target.mts')
  await import('./direct-target.mts')
  await import('./nested-target.mts')
  await import('./require-target.mts')
})
