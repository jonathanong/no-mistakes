import { write } from '@data-stores/psql'

export function stamp(id: string, at: string) {
  return write(`UPDATE orders SET updated_at = $2 WHERE id = $1`)
}
