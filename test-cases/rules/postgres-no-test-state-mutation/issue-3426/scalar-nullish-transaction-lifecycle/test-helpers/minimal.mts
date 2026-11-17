import { beginTransaction } from 'db'
import sql from 'sql-template-strings'

export async function nullSafeComparison(opts: { customer?: string | null }) {
  await using transaction = await beginTransaction()
  const query = transaction
  return query(sql`SELECT 1 WHERE c3 IS NOT DISTINCT FROM ${opts.customer ?? null}`)
}
