import { vi } from 'vitest'
import { buildMap } from './feature.mts'
vi.mock('./analysis.mts', () => ({ analyze: () => null }))
vi.mock('./analysis.mts', async () => ({ ...(await vi.importActual('./analysis.mts')) }))
buildMap()
