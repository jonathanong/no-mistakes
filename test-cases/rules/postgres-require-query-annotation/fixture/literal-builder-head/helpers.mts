import sql, { type SQLStatement } from 'sql-template-strings'

export function addImportedFilters(q: SQLStatement, id: string) {
  q.append(sql` AND x = ${id}`)
}
export function replaceImportedHead(q: SQLStatement) {
  q.strings[0] = 'SELECT 2'
}
