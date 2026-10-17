import sql, { type SQLStatement } from 'sql-template-strings'

export function buildCount(query: SQLStatement) {
  const count = sql`/* search/count */ SELECT COUNT(*) FROM (`
  count.append(query)
  count.append(sql`) AS matches`)
  return count
}
