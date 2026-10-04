import { write } from '@example/db'

export function noop(id: string) {
  return write(`INSERT INTO orders (id, status) VALUES ($1, 'new') ON CONFLICT (id) DO UPDATE SET updated_at = orders.updated_at RETURNING id`)
}
