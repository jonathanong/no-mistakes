// no-mistakes-disable-file test-no-unmocked-dynamic-imports: mock is intentionally local to this legacy test
import { test, vi } from 'vitest'

vi.mock('../src/leaf.mts')

test('legacy mock does not hide another test finding', async () => {
  await import('../src/leaf.mts')
  await import('../src/other.mts')
})
