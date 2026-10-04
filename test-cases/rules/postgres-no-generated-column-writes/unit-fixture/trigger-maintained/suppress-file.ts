// no-mistakes-disable-file postgres-no-generated-column-writes
import { write } from '@example/db'

export function file() {
  return write(`UPDATE orders SET updated_at = now()`)
}
