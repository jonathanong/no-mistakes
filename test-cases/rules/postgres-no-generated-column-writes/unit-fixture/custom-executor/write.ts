import { run } from '@app/db'

export function touch() {
  return run(`UPDATE items SET created_at = now()`)
}
