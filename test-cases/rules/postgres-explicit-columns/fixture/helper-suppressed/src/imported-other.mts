import sql from 'sql-template-strings'

export function otherImportedQuery() {
  return sql`SELECT * FROM accounts other_imported`
}
