import { vi } from 'vitest'
import { run } from './through.mts'
const mocks = vi.hoisted(() => make())
vi.mock('external-analyzer', () => ({ ...mocks }))
function make() {
  return { analyzeProject: vi.fn() }
}
run()
