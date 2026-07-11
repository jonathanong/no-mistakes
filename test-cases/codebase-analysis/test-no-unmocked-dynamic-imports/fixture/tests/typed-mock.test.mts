import { expect, test, vi } from 'vitest'

// Typed Vitest mock specifier (issue #506): `import(...)` here is a type carrier for the
// mocked module's shape, not a runtime dynamic import. It must cover the dependency the same
// way a string-literal specifier would.
vi.mock(import('@lib/typed-lazy.mts'), () => ({
  run: () => 'mocked',
}))

test('typed mock import specifier covers the dynamic import', async () => {
  const mod = await import('@lib/typed-lazy.mts')
  expect(mod.run()).toBe('mocked')
})
