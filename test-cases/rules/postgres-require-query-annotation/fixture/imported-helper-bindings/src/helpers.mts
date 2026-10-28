import sql from 'sql-template-strings'
export function named(value: unknown) { return sql`/* importedNamedBuilder */ SELECT ${value}` }
export function missing() { return sql`SELECT 1` }
export function plain() { return { text: '/* importedPlain */ SELECT 1', values: [] } }
export function returnedLocal(value: unknown) {
  const query = sql`/* importedReturnedLocal */ SELECT ${value}`
  query.append(sql` WHERE true`)
  return query
}
export function mutable() { return sql`/* mutableBuilder */ SELECT 1` }
mutable = missing
