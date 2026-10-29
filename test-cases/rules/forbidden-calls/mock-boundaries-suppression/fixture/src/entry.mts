import { vi } from 'vitest'
import { buildMap } from './feature.mts'
import { forbidden } from './forbidden.mts'
vi.mock('./analysis.mts', () => {
// no-mistakes-disable-next-line forbidden-calls
forbidden(); return { analyze: () => null }
})
buildMap()
