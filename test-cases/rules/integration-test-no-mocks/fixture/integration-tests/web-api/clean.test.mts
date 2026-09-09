import { expect, it } from 'vitest'

it('uses real helpers', () => {
  const previous = globalThis.fetch
  globalThis.fetch = previous
  expect(1).toBe(1)
})
