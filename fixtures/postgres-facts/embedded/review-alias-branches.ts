import type { TxExecutor } from '@example/db'

// A reassigned mutable alias may still refer to either earlier receiver.
export function branch(first: TxExecutor, other: TxExecutor, third: TxExecutor, flag: boolean) {
  let alias = first
  if (flag) alias = other
  sendAway(alias)
  first.query('SELECT id FROM branch_first')
  other.query('SELECT id FROM branch_other')
  third.query('SELECT id FROM branch_third')
}

export function opaque(first: TxExecutor, unknown: unknown, flag: boolean) {
  let alias = first
  if (flag) alias = unknown
  sendAway(alias)
  first.query('SELECT id FROM opaque_first')
}

export function self(tx: TxExecutor) {
  tx = tx
  tx.query('SELECT id FROM self_assignment')
}

export function loop(first: TxExecutor, other: TxExecutor) {
  let alias = first
  for (let i = 0; i < 2; i++) alias = other
  sendAway(alias)
  first.query('SELECT id FROM loop_first')
  other.query('SELECT id FROM loop_other')
}
