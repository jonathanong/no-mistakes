import { vi } from 'vitest'
import { buildMap } from './feature.mts'
vi.mock('./analysis.mts', () => ({ analyze: () => null }))
export function entry() { buildMap() }
