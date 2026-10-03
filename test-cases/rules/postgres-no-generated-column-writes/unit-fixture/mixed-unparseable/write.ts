import { write } from '@example/db'

export function touchCreatedAt() {
  return write(`UPDATE items SET created_at = now()`)
}
