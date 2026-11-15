import { write } from '@example/db'

export function insertVote() {
  return write(`INSERT INTO votes VALUES ($1, $2)`)
}
