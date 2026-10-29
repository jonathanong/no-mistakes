import { vi } from 'vitest'
import { buildMap } from './feature.mts'
vi.mock('./other-analysis.mts', () => ({ analyze: () => null }))
buildMap()
