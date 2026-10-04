import sql from 'sql-template-strings'
import { query } from '@data-stores/psql'

export function executedAndFragment(after: string) {
  const executed = sql`SELECT sql_placeholder_1 FROM orders WHERE id > ${after} ORDER BY id LIMIT 10`
  query(executed)
  return sql`SELECT ${after} FROM orders WHERE id > sql_placeholder_1 ORDER BY id LIMIT 10`
}
