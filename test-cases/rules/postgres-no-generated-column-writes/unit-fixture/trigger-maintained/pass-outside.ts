import { write } from '@example/db'

export function outside() {
  return write(`UPDATE missing SET updated_at = now()`)
}
