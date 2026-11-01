import { vi } from 'vitest'
import { run } from './through.mts'
const mocks = vi.hoisted(() => ({ analyzeProject: vi.fn(make()) }))
vi.mock('external-analyzer', () => ({ ...mocks }))
function make() {
  return () => ({})
}
run()
