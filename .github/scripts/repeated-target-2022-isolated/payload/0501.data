import { read } from '@example/db'
import { query } from './eager-builder'

const specifier = './missing-mutator'
require(specifier)

export function readAfterDynamicRequire() {
  return read(query)
}
