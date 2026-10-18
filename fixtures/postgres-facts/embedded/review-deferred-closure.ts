import type { TxExecutor } from '@example/db'

export function deferred(tx: TxExecutor) {
  tx.query('SELECT id FROM closure_before')
  // Lexical position precedes the escape, but execution may follow it.
  const later = () => tx.query('SELECT id FROM closure_deferred')
  sendAway(tx)
  return later()
}
