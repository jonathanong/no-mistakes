import type { TxExecutor } from '@example/db'

export function plain(run: TxExecutor) {
  return run('SELECT id FROM plain_param')
}

export function optional(run?: TxExecutor) {
  return run?.('SELECT id FROM optional_param')
}

export const arrow = (run: TxExecutor) => run('SELECT id FROM arrow_param')

export function nullable(run: TxExecutor | undefined) {
  return run?.('SELECT id FROM union_param')
}

export function destructured({ run }: { run: TxExecutor }) {
  return run('SELECT id FROM destructured_param')
}

// Not an executor: `run` is typed as a string; only `note` has the type.
export function otherProperty({ run, note }: { run: string; note: TxExecutor }) {
  return run('SELECT id FROM wrong_property')
}
