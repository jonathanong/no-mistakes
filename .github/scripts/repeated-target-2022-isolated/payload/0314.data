import { query, openTransaction, type TxExecutor } from '@example/db'

// With neither scoped option configured only the imported `query` is scanned.
export async function mixed(run: TxExecutor) {
  await query('SELECT id FROM imported_query')
  const tx = await openTransaction()
  tx('SELECT id FROM factory_default_off')
  return run('SELECT id FROM type_default_off')
}
