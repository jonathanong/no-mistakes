import { test } from 'vitest'

test('direct dynamic import is intentionally unmocked', async () => {
  // no-mistakes-disable-next-line test-no-unmocked-dynamic-imports: this direct import is intentional
  await import('../src/leaf.mts')
})
