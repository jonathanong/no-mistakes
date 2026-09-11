import sql, { type SQLStatement } from 'sql-template-strings'

export function build(): SQLStatement {
  return sql`EXISTS (`
}
