import { query } from "@example/db";

query("SELECT id FROM items");
// Malformed SQL must keep the historical conflict-policy result across modes.
query("SELECT (");

// The executed INSERT is inside a CTE; the outer statement is a SELECT.
query(`WITH inserted AS (
  INSERT INTO items (id)
  SELECT input.id FROM unnest($1::uuid[]) AS input(id)
  ON CONFLICT (id) DO NOTHING
  RETURNING id
) SELECT id FROM inserted`);

// Non-INSERT WITH prefixes must not demand conflict facts after pruning.
query("WITH v AS (SELECT id FROM items) SELECT id FROM v");
query("WITH (");
