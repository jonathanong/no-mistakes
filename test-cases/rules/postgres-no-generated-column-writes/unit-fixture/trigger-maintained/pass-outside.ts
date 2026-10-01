import { write } from '@data-stores/psql'

export function outside() {
  return write(`UPDATE missing SET updated_at = now()`)
}
