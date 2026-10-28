import { read } from '@bench/db'
import sql from 'sql-template-strings'

function composedCandidate() {
  return sql`/* max candidate */ SELECT 1`
}

export function maxOnly() {
  // This unrelated composed head forces max-only head collection in this file.
  let query = sql`SELECT 1`
  query = query.append(sql` WHERE TRUE`)
  read(query)
  read(composedCandidate())
}
