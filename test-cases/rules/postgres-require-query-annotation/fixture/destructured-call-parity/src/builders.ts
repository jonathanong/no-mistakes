import sql from 'sql-template-strings'

export function annotatedPageQuery(after: string, limit: number) {
  return sql`/* page */ SELECT 1 WHERE id > ${after} LIMIT ${limit}`
}

export function bareQuery(after: string) {
  return sql`SELECT 1 WHERE id > ${after}`
}
