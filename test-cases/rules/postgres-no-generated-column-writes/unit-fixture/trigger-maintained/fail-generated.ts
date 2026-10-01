import { write } from '@data-stores/psql'

export function gen() {
  return write(`UPDATE items SET created_at = now()`)
}
