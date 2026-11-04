import sql from 'sql-template-strings'

export function relation(limit: number) {
  return sql`
SELECT unknown_fn(id)
FROM accounts
LIMIT ${limit}
`
}
