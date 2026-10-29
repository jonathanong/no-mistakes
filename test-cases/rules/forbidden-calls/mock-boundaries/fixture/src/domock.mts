import { vi } from 'vitest'
import { buildMap } from './feature.mts'
vi.doMock('./analysis.mts', () => ({ analyze: () => null }))
buildMap()
