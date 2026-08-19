import { write } from '@data-stores/psql'

export function touchCreatedAt() {
  return write(`UPDATE items SET created_at = now()`)
}
