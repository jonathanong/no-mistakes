import { write } from '@example/db'

export function insertWithGeneratedCol() {
  return write(`INSERT INTO items (id, created_at, note) VALUES ($1, $2, $3)`)
}
