import sql, { type SQLStatement } from 'sql-template-strings'
import { read } from 'db'

export function parameter(query: SQLStatement) {
  query.append(sql` LIMIT 1`)
  return read(query.text, query.values)
}
