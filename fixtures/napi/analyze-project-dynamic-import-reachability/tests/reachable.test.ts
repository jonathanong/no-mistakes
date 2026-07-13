import { expect, test } from 'vitest'
import { loadReachable } from '../src/reachable.ts'

test('loads a reachable module', async () => {
  expect(await loadReachable()).toBe('lazy')
})
