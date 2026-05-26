import { test } from 'vitest'

test('next-line disable suppresses one import', async () => {
  // no-mistakes-disable-next-line test-no-unmocked-dynamic-imports
  await import('@lib/disabled.mts')
})
