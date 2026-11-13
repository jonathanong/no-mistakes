import sql from 'sql-template-strings'
import { read } from '@bench/db'
function labelSql() { return sql` AND label IS NOT NULL` }
function addArguments(q, filters, flag) {
  if (filters.length === 0) return
  q.append(flag ? sql` AND active` : sql` AND archived`)
  q.append(labelSql())
}
export function argumentsOnly(filters, flag) {
  const q = sql`/* arguments */ SELECT 1 WHERE true`
  addArguments(q, filters, flag)
  read(q)
}
