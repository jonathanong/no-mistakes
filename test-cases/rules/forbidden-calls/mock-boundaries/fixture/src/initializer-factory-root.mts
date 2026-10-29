import { vi } from 'vitest'
import { buildInitializer } from './initializer-feature.mts'

vi.mock('./initializer-module.mts', () => ({ analyzeInitializer: () => null }))
buildInitializer()
