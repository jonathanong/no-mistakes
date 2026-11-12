import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { appendFilters } from './i-filters-sql.mts'
export async function chained(filters, winners, page) {
  const query = sql`/* chained */ SELECT 1 FROM t WHERE x`
  appendFilters(query, filters)
  winners.forEach(filter => query.append(sql` AND `).append(filter))
  appendFilters(query, page)
  await read(query)
}
