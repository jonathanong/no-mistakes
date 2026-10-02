import sql from 'sql-template-strings'

// Equal text at distinct builder origins must share parsed facts, not finding locations.
export function first() {
  return sql`SELECT * FROM items WHERE id NOT IN (SELECT id FROM archived)`
}
export function second() {
  return sql`SELECT * FROM items WHERE id NOT IN (SELECT id FROM archived)`
}
