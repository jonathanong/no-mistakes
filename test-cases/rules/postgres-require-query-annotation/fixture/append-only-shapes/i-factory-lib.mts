import sql from 'sql-template-strings'
export function importedFactory(user) {
  return sql`/* importedFactory */ SELECT 1`
}
