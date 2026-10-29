import { buildMap } from './feature.mts'
const vi = { mock(...args: unknown[]) {} }
vi.mock('./analysis.mts', () => ({ analyze: () => null }))
buildMap()
