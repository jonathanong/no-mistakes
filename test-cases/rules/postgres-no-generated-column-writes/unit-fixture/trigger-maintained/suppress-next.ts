import { write } from '@data-stores/psql'

export function next() {
  // no-mistakes-disable-next-line postgres-no-generated-column-writes
  return write(`UPDATE orders SET updated_at = now()`)
}
