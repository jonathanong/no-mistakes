import { read } from '@example/db'
import sql from 'sql-template-strings'

export async function helper() {
  const query = buildQuery()
  await read(query)
}

function buildQuery() {
  return sql`SELECT execution.id FROM accounts a
    JOIN LATERAL (
      SELECT * FROM accounts attempt
      WHERE attempt.id = a.id
      ORDER BY attempt.id DESC LIMIT 1
    ) execution ON true`
}
