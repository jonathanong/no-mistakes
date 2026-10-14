import type { SQLStatement } from 'sql-template-strings'

export function appendUnknown(query: SQLStatement, runtimeSql: string): void {
  query.append(runtimeSql)
}
