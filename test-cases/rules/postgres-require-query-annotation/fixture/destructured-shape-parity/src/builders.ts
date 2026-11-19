import sql from 'sql-template-strings'

export function conditionalBuilder(id: string, owner?: string) {
  const statement = sql`/* byId */ SELECT id FROM widgets WHERE id = ${id}`
  if (owner) statement.append(sql` AND owner = ${owner}`)
  return statement
}

export function chainedBuilder(after: string | undefined, size: number) {
  const cursor = after ? sql` AND id > ${after}` : sql``
  return sql`/* page */
    SELECT id FROM widgets WHERE 1 = 1`.append(cursor).append(sql`
    LIMIT ${size}`)
}
