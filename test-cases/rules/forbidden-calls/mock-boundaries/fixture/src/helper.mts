import { vi } from 'vitest'
import { buildMap } from './feature.mts'
import { load } from './original-helper.mts'
vi.mock('./analysis.mts', () => { load(); return { analyze: () => null } })
buildMap()
