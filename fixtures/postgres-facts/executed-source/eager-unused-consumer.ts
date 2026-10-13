import { read } from '@example/db'
import { query } from './eager-builder'
import { unused } from './eager-mutator'

export function eagerRead() {
  return read(query)
}
