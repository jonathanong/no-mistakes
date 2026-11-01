import { vi } from 'vitest'
import { run } from './through.mts'
let mocks = vi.hoisted(() => ({ analyzeProject: vi.fn() }))
mocks = { analyzeProject: vi.fn() }
vi.mock('external-analyzer', () => ({ ...mocks }))
run()
