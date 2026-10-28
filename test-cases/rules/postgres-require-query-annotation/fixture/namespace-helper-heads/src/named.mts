import sql from 'sql-template-strings'
export default function source() {
  const statement = sql`/* namespaceNamed */ SELECT 1`
  statement.append(sql` WHERE true`)
  return statement
}
