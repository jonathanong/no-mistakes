import { vi } from 'vitest'
import { run } from './through.mts'
const mocks = vi.hoisted(() => ({ analyzeProject: vi.fn() }))
vi.mock('external-analyzer', () => (mocks))
run()
