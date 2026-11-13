import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { appendDescendantsCte } from '@services/a/descendants-sql'
export async function descendants(x, ids) {
  const query = sql`/* descendants */ SELECT 1 FROM t`
  appendDescendantsCte(query, { enabled: true, ids })
  let sortBy = sql`t.id DESC`
  if (x) sortBy = sql`rank DESC, t.id DESC`
  query.append(sql` ORDER BY `)
  query.append(sortBy)
  await read(query)
}
