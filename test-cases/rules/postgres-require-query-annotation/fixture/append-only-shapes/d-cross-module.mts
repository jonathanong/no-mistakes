import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { addColumns } from './columns.mts'
export function columns(status) {
  const q = sql`/* columns */ SELECT 1 WHERE true`
  addColumns(q, status)
  read(q)
}
