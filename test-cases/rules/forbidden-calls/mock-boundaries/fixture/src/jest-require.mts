import { buildMap } from './feature.mts'
jest.mock('./analysis.mts')
require('node:fs')
buildMap()
