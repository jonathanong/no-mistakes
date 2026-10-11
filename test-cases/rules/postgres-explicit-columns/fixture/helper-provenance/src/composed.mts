import { read } from '@example/db'
import sql from 'sql-template-strings'

function first() {
  return sql`SELECT a.id FROM accounts a
    JOIN LATERAL (
      SELECT * FROM accounts first
      WHERE first.id = a.id
    ) f ON true`
}

function both() {
  const query = first()
  query.append(sql` UNION ALL SELECT a.id FROM accounts a
    JOIN LATERAL (
      SELECT * FROM accounts second
      WHERE second.id = a.id
    ) s ON true`)
  return query
}

export async function execute() {
  const assigned = both()
  await read(assigned)
}
