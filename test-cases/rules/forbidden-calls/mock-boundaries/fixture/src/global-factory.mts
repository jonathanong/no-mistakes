import { vi } from 'vitest'
import { buildMap } from './feature.mts'
vi.mock('./analysis.mts', () => { loadReal(); return { analyze: () => null } })
buildMap()
