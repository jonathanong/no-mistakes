import { vi } from 'vitest'
import { analyze } from './analysis.mts'
vi.mock('./analysis.mts')
analyze()
