import type { TxExecutor } from '@example/db'
import { openTransaction } from '@example/db'

// Escaping one binding cannot erase calls on an independent typed parameter.
export function typed(first: TxExecutor, other: TxExecutor) {
  first.query('SELECT id FROM typed_before')
  const alias = first
  sendAway(alias)
  first.query('SELECT id FROM typed_after')
  alias.query('SELECT id FROM typed_alias_after')
  other.query('SELECT id FROM typed_other')
}

// Factory results have the same per-binding escape behavior.
export function factory() {
  const first = openTransaction()
  const other = openTransaction()
  first.query('SELECT id FROM factory_before')
  sendAway(first)
  first.query('SELECT id FROM factory_after')
  other.query('SELECT id FROM factory_other')
}
