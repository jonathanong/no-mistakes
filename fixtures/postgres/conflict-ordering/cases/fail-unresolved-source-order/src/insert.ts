import { query } from '@data-stores/psql'

export function insert() {
  // Omitting INSERT target columns makes the source-to-arbiter mapping unknown.
  return query(`
    INSERT INTO items
    SELECT id FROM pending_items ORDER BY id
    ON CONFLICT (id) DO NOTHING
  `)
}
