import type { TxExecutor } from '@example/db/types'
import { openTransaction } from '@example/db/tx/open'
import { type TxExecutor as Exec } from '@example/db/types'

// Subpaths of `importSpecifier` count as the configured module.
export async function subpaths(run: TxExecutor, aliased: Exec) {
  const tx = await openTransaction()
  tx('SELECT id FROM subpath_factory')
  run('SELECT id FROM subpath_type')
  return aliased('SELECT id FROM subpath_inline_type')
}
