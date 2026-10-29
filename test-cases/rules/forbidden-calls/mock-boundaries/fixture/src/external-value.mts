import { vi } from 'vitest'
import { fake } from 'opaque-pkg'
import { buildMap } from './feature.mts'
vi.mock('./analysis.mts', () => ({ analyze: () => fake.value }))
buildMap()
