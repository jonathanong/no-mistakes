import { vi } from 'vitest'
import { build } from './automock-initializers-feature.mts'

vi.mock('./automock-initializers-target.mts')
build()
