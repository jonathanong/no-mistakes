import { openTransaction } from '@example/db'

export async function viaMember() {
  await using tx = await openTransaction()
  await tx.query('SELECT id FROM member_query')
  return tx['query']('SELECT id FROM computed_query')
}
