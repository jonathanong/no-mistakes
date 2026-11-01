import { vi } from 'vitest'
import { run } from './through.mts'
const plain = { analyzeProject: vi.fn() }
vi.mock('external-analyzer', () => ({ ...plain }))
run()
