import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { appendDescendantsCte } from '@services/a/descendants-sql'
export async function combined(cond, ids, filters, sortBy) {
  const query = sql`/* combined */ SELECT 1 FROM t`
  query.append(cond ? sql`WITH RECURSIVE ` : sql`WITH `)
  appendDescendantsCte(query, { enabled: true, ids })
  filters.forEach((filter, index) => {
    if (index > 0) query.append(sql` AND `)
    query.append(filter)
  })
  query.append(sql` ORDER BY `)
  query.append(sortBy)
  await read(query)
}
