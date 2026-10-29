import { vi } from 'vitest'
import { buildMap } from './feature.mts'
import * as actual from './analysis.mts'
vi.mock('./analysis.mts', () => actual)
buildMap()
