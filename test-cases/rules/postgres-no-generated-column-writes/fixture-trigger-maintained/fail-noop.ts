import { write } from '@data-stores/psql'

export function noop(id: string) {
  return write(`INSERT INTO orders (id, status) VALUES ($1, 'new') ON CONFLICT (id) DO UPDATE SET updated_at = orders.updated_at RETURNING id`)
}
