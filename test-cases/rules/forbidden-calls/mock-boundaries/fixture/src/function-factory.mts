import { vi } from 'vitest'
import { buildMap } from './feature.mts'
import { forbidden } from './forbidden.mts'
vi.mock('./analysis.mts', () => { forbidden(); return { analyze: () => null } })
export function entry() { buildMap() }
