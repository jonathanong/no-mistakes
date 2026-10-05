import { openTransaction, type TxExecutor } from '@example/db'

export function declares(run: TxExecutor) {
  return run('BEGIN')
}

// Counterintuitive on purpose: same name as the typed parameter above, but
// this function's `run` is untyped, so it must not be reported.
export function sibling(run: (sql: string) => unknown) {
  return run('BEGIN')
}

export async function block(flag: boolean) {
  if (flag) {
    const tx = await openTransaction()
    tx('BEGIN')
  }
  const tx = (sql: string) => sql
  // Same name as the block-scoped factory result, but outside that block.
  return tx('BEGIN')
}

// A destructured factory result is not tracked, and neither is a non-call.
export async function untracked() {
  const { tx } = await openTransaction()
  const other = openTransaction
  tx('BEGIN')
  return other('BEGIN')
}
