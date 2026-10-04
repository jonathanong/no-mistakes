import { write } from '@example/db'

export function next() {
  // no-mistakes-disable-next-line postgres-no-generated-column-writes
  return write(`UPDATE orders SET updated_at = now()`)
}
