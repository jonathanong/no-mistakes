import { write } from '@example/db'

export function insert(id: string) {
  return write(`INSERT INTO orders (id, status, updated_at) VALUES ($1, 'new', now())`)
}
