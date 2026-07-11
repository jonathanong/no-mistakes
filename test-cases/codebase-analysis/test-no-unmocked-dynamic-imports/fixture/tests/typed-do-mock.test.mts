import { expect, test, vi } from 'vitest'

// Same as typed-mock.test.mts but via `vi.doMock`, which must be recognized identically.
vi.doMock(import('@lib/typed-do-lazy.mts'), () => ({
  run: () => 'mocked',
}))

test('typed doMock import specifier covers the dynamic import', async () => {
  const mod = await import('@lib/typed-do-lazy.mts')
  expect(mod.run()).toBe('mocked')
})
