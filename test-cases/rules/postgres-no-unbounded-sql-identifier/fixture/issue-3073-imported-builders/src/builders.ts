import sql, { type SQLStatement } from 'sql-template-strings'

export const statement = sql`id, url`.append(sql` FROM items WHERE active = ${true}`)
export const fragment = sql`id, url`

export function buildRead(ids: string[]): SQLStatement {
  return sql`id, url FROM items WHERE id = ANY(${ids}::uuid[])`
}

export function buildLocal(id: string): SQLStatement {
  const result = sql`id, url FROM items WHERE id = ${id}`
  result.append(sql` LIMIT ${10}`)
  return result
}
