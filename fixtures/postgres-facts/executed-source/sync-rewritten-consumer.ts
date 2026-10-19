import { read } from '@example/db'
import { buildSql } from './sync-rewritten-builder.ts'

export function directRead() {
  return read(buildSql())
}
