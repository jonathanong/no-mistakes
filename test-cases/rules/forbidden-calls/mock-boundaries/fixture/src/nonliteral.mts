import { vi } from 'vitest'
import { buildMap } from './feature.mts'
const target = './analysis.mts'
vi.mock(target, () => ({ analyze: () => null }))
buildMap()
