import type { TxExecutor } from '@example/dbx'
import { openTransaction } from '@example/db-utils'
import type { TxExecutor as Other } from '@example/db-utils/types'

// Counterintuitive on purpose: these modules only share a string prefix with
// `@example/db` (no `/` boundary), so nothing is scanned.
export async function lookalikes(run: TxExecutor, other: Other) {
  const tx = await openTransaction()
  tx('SELECT id FROM lookalike_factory')
  run('SELECT id FROM lookalike_type')
  return other('SELECT id FROM lookalike_subpath')
}
