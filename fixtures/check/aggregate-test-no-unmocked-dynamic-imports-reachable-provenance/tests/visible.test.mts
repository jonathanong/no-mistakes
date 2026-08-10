import { test } from 'vitest'
import { loadHelper } from '../src/helper.mts'

test('visible helper reachability', async () => {
  await loadHelper()
})
