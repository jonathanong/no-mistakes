import { query } from '@data-stores/psql'

export function insertItem() {
  return query('INSERT INTO items (id) VALUES (1) ON CONFLICT (id) DO NOTHING')
}
