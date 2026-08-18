import { write } from '@data-stores/psql'

export function insertWithGeneratedCol() {
  return write(`INSERT INTO items (id, created_at, note) VALUES ($1, $2, $3)`)
}
