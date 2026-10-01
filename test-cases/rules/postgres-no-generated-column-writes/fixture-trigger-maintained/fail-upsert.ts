import { write } from '@data-stores/psql'

export function upsert(id: string) {
  return write(`INSERT INTO orders (id, status) VALUES ($1, 'new') ON CONFLICT (id) DO UPDATE SET status = EXCLUDED.status, updated_at = CURRENT_TIMESTAMP`)
}
