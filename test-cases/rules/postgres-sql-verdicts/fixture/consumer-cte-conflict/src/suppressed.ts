import { query } from '@example/db'

// An explicit line directive hides the finding while retaining raw proof evidence.
// no-mistakes-disable-next-line postgres-conflict-ordering
query(`WITH inserted AS (
  INSERT INTO items (id)
  SELECT input.id FROM unnest($1::uuid[]) AS input(id)
  ON CONFLICT (id) DO NOTHING
  RETURNING id
) SELECT id FROM inserted`)
