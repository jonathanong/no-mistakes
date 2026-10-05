import { query, openTransaction, type TxExecutor } from '@example/db'

// With neither scoped option configured only the imported `query` is reported.
export async function mixed(run: TxExecutor) {
  await query('BEGIN')
  const tx = await openTransaction()
  tx('BEGIN')
  return run('BEGIN')
}
