import sql, { type SQLStatement } from 'sql-template-strings'
import { read } from 'db'
declare function escape(value: unknown): void
declare function opaque(): SQLStatement

export function rawAppend(table: string) {
  const rawStatement = sql``
  rawStatement.append(table)
  return read(`SELECT ${rawStatement.text}`, rawStatement.values)
}
export function mutatedBuilder(table: string) {
  const mutatedStatement = sql`id FROM items`
  mutatedStatement.text = table
  return read(`SELECT ${mutatedStatement.text}`, mutatedStatement.values)
}
export function escapedBuilder(table: string) {
  const escapedStatement = sql`id FROM items WHERE id = ${table}`
  escape(escapedStatement)
  return read(`SELECT ${escapedStatement.text}`, escapedStatement.values)
}
export function conditionalRaw(flag: boolean, table: string) {
  const conditionalStatement = sql``.append(sql`id FROM items`)
  if (flag) conditionalStatement.append(table)
  return read(`SELECT ${conditionalStatement.text}`, conditionalStatement.values)
}
export function rawChain(table: string) {
  const rawChainStatement = sql``.append(table)
  return read(`SELECT ${rawChainStatement.text}`, rawChainStatement.values)
}
function unsafeHelper(table: string): SQLStatement {
  return sql``.append(table)
}
export function rawHelper(table: string) {
  const helperStatement = unsafeHelper(table)
  return read(`SELECT ${helperStatement.text}`, helperStatement.values)
}
export function unknownBuilder() {
  const unknownStatement = opaque()
  return read(`SELECT ${unknownStatement.text}`, unknownStatement.values)
}
export function shadowedTag(sql: (parts: TemplateStringsArray) => SQLStatement) {
  const shadowedStatement = sql`id FROM items`
  return read(`SELECT ${shadowedStatement.text}`, shadowedStatement.values)
}
export function rawMemberMutation(table: string) {
  const memberStatement = sql`id FROM items`
  memberStatement.strings.push(table)
  return read(`SELECT ${memberStatement.text}`, memberStatement.values)
}
const escapedColumns = sql`id FROM items`
escape(escapedColumns)
export function escapedFragment() {
  const fragmentStatement = sql``.append(escapedColumns)
  return read(`SELECT ${fragmentStatement.text}`, fragmentStatement.values)
}
export function unknownAppendReceiver() {
  const columns = sql`id FROM items`
  const receiver = opaque()
  receiver.append(columns)
  return read(`SELECT ${columns.text}`, columns.values)
}
export function mutatedAppendReceiver(table: string) {
  const mutatedColumns = sql`id FROM items`
  const receiver = sql``
  receiver.append = () => { mutatedColumns.text = table; return receiver }
  receiver.append(mutatedColumns)
  return read(`SELECT ${mutatedColumns.text}`, mutatedColumns.values)
}
