import { query } from '@data-stores/psql'

// Both rules consume distinct SQL occurrences from this one prepared AST.
await query(`
  INSERT INTO items (id)
  SELECT id FROM pending_items ORDER BY id
  ON CONFLICT (id) DO NOTHING
`)

await query(`
  SELECT id FROM items WHERE id = ANY($1) ORDER BY id FOR UPDATE
`)
