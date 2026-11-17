import { beginTransaction } from '@data-stores/psql'
import sql from 'sql-template-strings'

export async function forbiddenMutation() {
  await using transaction = await beginTransaction()
  const query = transaction
  await query(sql`TRUNCATE TABLE t`)
}
