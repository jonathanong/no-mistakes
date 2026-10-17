import sql, { type SQLStatement } from 'sql-template-strings'

export function appendSearchCte(query: SQLStatement): void {
  query.append(sql`WITH eligible AS (SELECT id FROM posts) `)
}
