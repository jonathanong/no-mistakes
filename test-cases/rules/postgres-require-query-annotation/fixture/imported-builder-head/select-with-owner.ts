import sql, { type SQLStatement } from 'sql-template-strings'

export function selectWithOwner(select: SQLStatement): SQLStatement {
  const statement = select.append(sql`c.id, c.name`).append(', u.id AS owner_id FROM items c JOIN users u ON u.id = c.owner_id')
  return statement
}
