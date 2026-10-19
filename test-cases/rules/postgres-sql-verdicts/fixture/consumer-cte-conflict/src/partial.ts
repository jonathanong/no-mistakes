import { query } from '@example/db'
declare const choose: boolean

// The incomplete alternative must not borrow a proof from its recovered CTE.
query(choose
  ? `WITH inserted AS (
      INSERT INTO items (id) VALUES (1)
      ON CONFLICT (id) DO NOTHING RETURNING id
    ) SELECT id FROM inserted`
  : `WITH inserted AS (
      INSERT INTO items (id)
      SELECT input.id FROM unnest($1::uuid[]) AS input(id)
      ON CONFLICT (id) DO NOTHING RETURNING id
    ) SELECT id FROM inserted WHERE (`)
