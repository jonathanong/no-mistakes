import { query } from '@example/db'
import sql from 'sql-template-strings'

export function executed() {
  return query('SELECT id FROM accounts WHERE id = $1')
}
// Unexecuted builders retain shape facts without asking for row-bound facts.
export function fragment() {
  return sql`SELECT * FROM accounts WHERE id NOT IN (SELECT id FROM archived)`
}
