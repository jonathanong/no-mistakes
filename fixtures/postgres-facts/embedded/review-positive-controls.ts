import { openTransaction, type TxExecutor } from '@example/db'

// This file proves the same configuration recognizes unescaped receivers.
export function typed(tx: TxExecutor) {
  tx.query('SELECT id FROM control_typed')
}

export function factory() {
  const tx = openTransaction()
  tx.query('SELECT id FROM control_factory')
}
