import { read } from '@example/db'
import { query } from './eager-builder'
import './eager-mutator'

export function eagerRead() {
  return read(query)
}
