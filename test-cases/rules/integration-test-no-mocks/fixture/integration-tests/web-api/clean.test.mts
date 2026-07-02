import { expect, it } from 'vitest'

// vi.mock('commented-policy-example') is prose, not a mock call.
it('uses real helpers', () => {
  const previous = globalThis.fetch
  globalThis.fetch = previous
  expect(1).toBe(1)
})
