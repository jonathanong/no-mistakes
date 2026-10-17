import { openTransaction, type TxExecutor } from '@example/db'

export function declares(run: TxExecutor) {
  return run('SELECT id FROM typed_here')
}

// Counterintuitive on purpose: same name as the typed parameter above, but
// this function's `run` is untyped, so it must not be scanned.
export function sibling(run: (sql: string) => unknown) {
  return run('SELECT id FROM sibling_function')
}

export async function block(flag: boolean) {
  if (flag) {
    const tx = await openTransaction()
    tx('SELECT id FROM inside_block')
  }
  const tx = (sql: string) => sql
  // Same name as the block-scoped factory result, but outside that block.
  return tx('SELECT id FROM outside_block')
}

export function otherFunction(tx: (sql: string) => unknown) {
  return tx('SELECT id FROM other_function')
}
