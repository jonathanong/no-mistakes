import type { TxExecutor } from '@example/db'

// The parameter spelling is identical; declaration scopes are independent.
export function escaped(tx: TxExecutor) {
  sendAway(tx)
  return tx.query('SELECT id FROM sibling_escaped')
}

export function untouched(tx: TxExecutor) {
  return tx.query('SELECT id FROM sibling_untouched')
}
