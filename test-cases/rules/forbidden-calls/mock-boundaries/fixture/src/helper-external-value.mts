import { vi } from 'vitest'
import { buildMap } from './feature.mts'
import { getFake } from './value-helper.mts'
vi.mock('./analysis.mts', () => { getFake(); return { analyze: () => null } })
buildMap()
