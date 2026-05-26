import { test } from 'vitest'

test('jest setup mocks do not apply to vitest tests', async () => {
  await import('@lib/jest-setup-target.mts')
})
