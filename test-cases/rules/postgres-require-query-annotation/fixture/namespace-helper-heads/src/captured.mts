import sql from 'sql-template-strings'
export default function source() {
  const statement = sql`/* capturedRoot */ SELECT 1`;
  // The callback may run outside the straight-line return proof.
  [1].forEach(() => { statement.append(sql` WHERE true`) })
  return statement
}
