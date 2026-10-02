import { write } from '@data-stores/psql'

export function ok(id: string, seenAt: string) {
  write(`UPDATE orders SET status = 'paid' WHERE id = $1`)
  write(`UPDATE orders SET status = 'paid' WHERE id = $1 AND updated_at = $2 RETURNING updated_at`)
  write(`SELECT id FROM orders ORDER BY updated_at DESC LIMIT 20`)
  return write(`INSERT INTO orders (id, status) VALUES ($1, 'new') ON CONFLICT (id) DO NOTHING`)
}
