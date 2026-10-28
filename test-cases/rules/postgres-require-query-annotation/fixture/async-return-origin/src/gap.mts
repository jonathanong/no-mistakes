import sql from 'sql-template-strings'
export async function gap() {
  const query = sql`/* gap */ SELECT 1`
  const query = sql`/* duplicate */ SELECT 1`
  return query
}
