import { write } from '@data-stores/psql'

export function update() {
  return write(`UPDATE items SET note = $1`)
}
