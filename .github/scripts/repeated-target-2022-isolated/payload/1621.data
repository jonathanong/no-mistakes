import { openTransaction } from '@other/db'
import type { TxExecutor } from '@third/db'

// Without `importSpecifier`, factory and type imports match from any module.
export async function anyModule(run: TxExecutor) {
  const tx = await openTransaction()
  tx('SELECT id FROM any_module_factory')
  return run('SELECT id FROM any_module_type')
}
