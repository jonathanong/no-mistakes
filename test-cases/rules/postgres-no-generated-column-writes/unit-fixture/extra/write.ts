import { write } from '@data-stores/psql'

export function touchVote() {
  return write(`UPDATE votes SET created_at = now()`)
}
