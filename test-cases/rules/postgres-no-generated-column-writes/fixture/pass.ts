import { write } from '@data-stores/psql'

export function insertSourceColumn() {
  return write(`INSERT INTO items (id, note) VALUES ($1, $2)`)
}

export function updateSourceColumn() {
  return write(`UPDATE items SET note = $1`)
}
