import { vi } from 'vitest'
import { run } from './through.mts'
const helper = { fn: (value) => value }
vi.mock('external-analyzer', () => ({ analyzeProject: helper.fn(() => ({})) }))
run()
