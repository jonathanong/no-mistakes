import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { appendFilters } from './i-filters-sql.mts'
export async function filtered(filters) {
  const query = sql`/* filtered */ SELECT 1 FROM t`
  appendFilters(query, filters)
  await read(query)
}
