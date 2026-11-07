import sql from 'sql-template-strings'
import { read } from 'db'

export function appendAfterDeclaration(id: string, extra: boolean) {
  const appendedStatement = sql`id FROM items WHERE id = ${id}`
  if (extra) appendedStatement.append(sql` AND active = ${true}`)
  appendedStatement.append(sql` ORDER BY id LIMIT ${10}`)
  return read(`SELECT ${appendedStatement.text}`, appendedStatement.values)
}
