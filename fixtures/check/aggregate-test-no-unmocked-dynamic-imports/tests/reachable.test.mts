import { test } from 'vitest'
import { loadReachable } from '../src/reachable.mts'

test('reachable dynamic import is intentionally unmocked', () => {
  loadReachable()
})
