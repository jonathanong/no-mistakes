import { write } from '@example/db'

export function line() {
  return write(`UPDATE orders SET updated_at = now()`) // no-mistakes-disable-line postgres-no-generated-column-writes
}
