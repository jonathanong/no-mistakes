import sql from 'sql-template-strings'
export async function dynamic() {
  const query = sql`/* dynamic */ SELECT 1`
  query.append(eval('suffix'))
  return query
}
