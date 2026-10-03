import { write } from '@example/db'

export function insertColumnless() {
  return write(`INSERT INTO items VALUES ($1, $2, $3)`)
}
