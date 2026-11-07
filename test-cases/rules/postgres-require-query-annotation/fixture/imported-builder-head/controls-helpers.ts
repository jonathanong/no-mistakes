import sql, { type SQLStatement } from 'sql-template-strings'
import { selectWithOwner } from './select-with-owner.ts'

export function strings(select: SQLStatement) {
  const statement = select.append('id').append(' FROM items')
  return statement
}
export function dynamic(select: SQLStatement) {
  const statement = select.append(sql`id ${unknownValue()}`)
  return statement
}
export function forwarded(select: SQLStatement) {
  return selectWithOwner(select)
}
