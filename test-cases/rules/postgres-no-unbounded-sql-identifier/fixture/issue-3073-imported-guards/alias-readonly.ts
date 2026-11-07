import { write } from 'db'
import { ALLOWED_TABLES as tables } from './tables.ts'
import * as namespace from './tables.ts'
type Allowed = (typeof tables)[number]
export function readonlyMembership(table: Allowed) {
  void namespace.ALLOWED_TABLES.length
  void namespace.ALLOWED_TABLES.includes(table)
  if (tables.length === 0 || !tables.includes(table)) throw new Error('bad')
  if (!tables.includes(table)) throw new Error('bad')
  return write(`/* readonlyMembership */ SELECT * FROM ${table}`)
}
