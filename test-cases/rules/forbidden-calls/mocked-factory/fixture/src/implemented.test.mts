import { vi } from 'vitest'
import { analyzeProject } from 'external-analyzer'
import { run } from './through.mts'
vi.mock('external-analyzer')
vi.mocked(analyzeProject).mockResolvedValue({})
run()
