import type { TxExecutor } from '@example/db'

export function plain(run: TxExecutor) {
  return run('BEGIN')
}

export function optional(run?: TxExecutor) {
  return run?.('BEGIN')
}

export const arrow = (run: TxExecutor) => run('BEGIN')

export function nullable(run: TxExecutor | undefined) {
  return run?.('BEGIN')
}

export function defaulted(run: TxExecutor = fallback) {
  return run('BEGIN')
}

export function destructured({ run }: { run: TxExecutor }) {
  return run('BEGIN')
}

export function destructuredDefault({ run = fallback }: { run?: TxExecutor }) {
  return run('BEGIN')
}

// Not an executor: `run` is typed as a string; only `note` has the type.
export function otherProperty({ run, note }: { run: string; note: TxExecutor }) {
  return run('BEGIN')
}

// Not an executor: no inline type literal to read the property type from.
export function untypedPattern({ run }: Options, [first]: TxExecutor[], ...rest: TxExecutor[]) {
  return run('BEGIN')
}
