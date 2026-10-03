import { write } from '@example/db'

export function both() {
  return write(`UPDATE orders SET updated_at = now(), modified_at = now()`)
}
