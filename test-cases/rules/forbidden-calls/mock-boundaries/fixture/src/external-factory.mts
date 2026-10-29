import { vi } from 'vitest'
import { buildMap } from './feature.mts'
import { load } from 'opaque-pkg'
vi.mock('./analysis.mts', () => { load(); return { analyze: () => null } })
buildMap()
