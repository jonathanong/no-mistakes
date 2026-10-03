import { write } from '@example/db'

export function twice(id: string) {
  return write(`INSERT INTO orders (id, updated_at) VALUES ($1, now()) ON CONFLICT (id) DO UPDATE SET updated_at = EXCLUDED.updated_at`)
}
