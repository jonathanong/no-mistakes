import { write } from '@example/db'

export function upsertCreatedAt() {
  return write(
    `INSERT INTO items (id, note) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET created_at = now()`,
  )
}
