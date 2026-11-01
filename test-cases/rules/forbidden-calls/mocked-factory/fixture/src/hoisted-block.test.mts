import { vi } from 'vitest'
import { run } from './through.mts'
const mocks = vi.hoisted(() => {
  return { analyzeProject: vi.fn(1) }
})
vi.mock('external-analyzer', () => {
  return { ...mocks, default: mocks } satisfies object
})
run()
