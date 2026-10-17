import { type TxExecutor as Exec } from '@example/db'

// `Exec` is a local alias of the configured type name `TxExecutor`.
export function aliased(run: Exec) {
  return run('SELECT id FROM inline_type_specifier')
}
