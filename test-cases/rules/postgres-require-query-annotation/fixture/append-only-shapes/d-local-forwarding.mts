import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { addColumns } from './columns.mts'

function addLocally(q, status) {
  addColumns(q, status)
}
export function localForwarding(status) {
  const q = sql`/* local forwarding */ SELECT 1 WHERE true`
  addLocally(q, status)
  read(q)
}
