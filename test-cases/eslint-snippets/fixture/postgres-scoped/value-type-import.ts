import { TxExecutor, openTransaction } from '@example/db'
import { type openTransaction as typeOnlyFactory } from '@example/db'

// A value import of the configured type name still types the parameter, but a
// type-only import of the factory is not callable and must not bind.
export async function mixed(run: TxExecutor) {
  const tx = await typeOnlyFactory()
  tx('BEGIN')
  return run('BEGIN')
}
