import { write } from '@example/db'

export function update() {
  return write(`UPDATE items SET note = $1`)
}
