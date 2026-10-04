import { write } from '@example/db'

export function touchVote() {
  return write(`UPDATE votes SET created_at = now()`)
}
