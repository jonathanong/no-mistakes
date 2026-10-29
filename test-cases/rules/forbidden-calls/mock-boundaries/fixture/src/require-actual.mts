import { jest } from '@jest/globals'
import { buildMap } from './feature.mts'
jest.mock('./analysis.mts', () => ({ ...jest.requireActual('./analysis.mts') }))
buildMap()
