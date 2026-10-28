import sql from 'sql-template-strings'
export async function mutated() {
  const query = sql`/* mutated */ SELECT 1`
  return query
}
mutated = replacement
