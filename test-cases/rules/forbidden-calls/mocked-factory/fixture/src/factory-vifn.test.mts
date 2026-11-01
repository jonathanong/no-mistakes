import { vi } from 'vitest'
import { run } from './through.mts'
vi.mock('external-analyzer', () => ({ analyzeProject: vi.fn(async () => ({})) }))
run()
