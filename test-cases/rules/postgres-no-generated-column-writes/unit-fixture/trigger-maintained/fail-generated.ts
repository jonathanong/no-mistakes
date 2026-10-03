import { write } from '@example/db'

export function gen() {
  return write(`UPDATE items SET created_at = now()`)
}
