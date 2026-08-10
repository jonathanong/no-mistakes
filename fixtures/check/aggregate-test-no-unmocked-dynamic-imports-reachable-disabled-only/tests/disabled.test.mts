// no-mistakes-disable-file test-no-unmocked-dynamic-imports: legacy test is intentionally suppressed
import { test } from 'vitest'
import { loadHelper } from '../src/helper.mts'

test('legacy helper reachability', async () => {
  await loadHelper()
})
