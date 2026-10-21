import type { TxExecutor } from '@example/db'

export function quoted({ "run": run }: { "run": TxExecutor }) {
  return run('BEGIN')
}

export function quotedDefault({ "run": run = fallback }: { "run"?: TxExecutor }) {
  return run('BEGIN')
}
