import { vi } from 'vitest'
import { run } from './through.mts'
vi.mock('external-analyzer', () => ({ analyzeProject: () => synthetic() }))
function synthetic() {
  return {}
}
run()
