import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { appendForeign as first, appendForeign as second } from './helpers.mts'
function combined(q) { first(q) }
export function rejected() {
  const q = sql`/* duplicate */ SELECT 1`
  combined(q)
  read(q) // low: false
}
