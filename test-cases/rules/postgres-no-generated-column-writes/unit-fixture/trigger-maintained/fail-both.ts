import { write } from '@data-stores/psql'

export function both() {
  return write(`UPDATE orders SET updated_at = now(), modified_at = now()`)
}
