import { openTransaction } from '@other/db'
import type { TxExecutor } from '@other/db'

// Counterintuitive on purpose: the names match the configuration but the
// module does not match `importSpecifier`, so nothing is reported.
export async function wrongModule(run: TxExecutor) {
  const tx = await openTransaction()
  tx('BEGIN')
  return run('BEGIN')
}
