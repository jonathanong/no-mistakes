import { analyzeProject } from 'no-mistakes'
import { vi } from 'vitest'
vi.mock('no-mistakes')
analyzeProject()
function localHelper() {
  analyzeProject()
}
localHelper()
