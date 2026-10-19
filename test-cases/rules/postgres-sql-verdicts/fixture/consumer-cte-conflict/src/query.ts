import { query } from '@example/db'

// The executed INSERT is inside a CTE; the outer statement is a SELECT.
query(`WITH inserted AS (
  INSERT INTO items (id)
  SELECT input.id FROM unnest($1::uuid[]) AS input(id)
  ON CONFLICT (id) DO NOTHING
  RETURNING id
) SELECT id FROM inserted`)
