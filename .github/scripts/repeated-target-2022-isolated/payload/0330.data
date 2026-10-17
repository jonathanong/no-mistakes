import { read } from '@example/db'
import sql from 'sql-template-strings'

function base() {
  return sql`SELECT a.id FROM accounts a
    JOIN LATERAL (
      SELECT * FROM accounts first
      WHERE first.id = a.id
    ) f ON true`
}

function withAppend() {
  const query = base()
  query.append(sql` UNION ALL SELECT a.id FROM accounts a
    JOIN LATERAL (
      SELECT * FROM accounts second
      WHERE second.id = a.id
    ) s ON true`)
  return query
}

export async function run() {
  const assigned = withAppend()
  await read(assigned)
  await read(sql`SELECT * FROM accounts`)
  await read(base())
}
