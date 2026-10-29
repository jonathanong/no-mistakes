import { vi } from 'vitest'
import { buildDependency } from './initializer-dependency-feature.mts'

vi.mock('./initializer-dependency-module.mts')
buildDependency()
