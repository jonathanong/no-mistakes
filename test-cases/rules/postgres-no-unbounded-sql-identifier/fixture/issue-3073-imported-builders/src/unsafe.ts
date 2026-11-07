import sql, { type SQLStatement } from 'sql-template-strings'
declare function escape(value: unknown): void

export const rawStatement = sql``.append(process.env.TABLE)
export const mutatedStatement = sql`id FROM items`
mutatedStatement.text = process.env.TABLE
export const escapedStatement = sql`id FROM items`
escape(escapedStatement)
export const consumerMutated = sql`id FROM items`

export function buildRaw(table: string): SQLStatement {
  return sql``.append(table)
}
export function buildEscaped(): SQLStatement {
  const result = sql`id FROM items`
  escape(result)
  return result
}
