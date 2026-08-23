import { write } from '@data-stores/psql'

export function insertColumnless() {
  return write(`INSERT INTO items VALUES ($1, $2, $3)`)
}
