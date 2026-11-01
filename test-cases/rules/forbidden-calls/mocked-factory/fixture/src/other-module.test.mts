import { vi } from 'vitest'
import { run } from './through.mts'
vi.mock('other-analyzer', () => ({ analyzeProject: vi.fn() }))
run()
