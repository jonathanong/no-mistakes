import { vi } from 'vitest'
import { buildMap } from './feature.mts'
vi.mock('./analysis.mts', () => ({ analyze: () => null }))
await vi.importActual('./other-analysis.mts')
buildMap()
