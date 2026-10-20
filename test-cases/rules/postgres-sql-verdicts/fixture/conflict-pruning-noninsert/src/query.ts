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

// Later-line arguments must keep the same policy result when their facts are omitted.
query(
  "SELECT account_id, tenant_id, created_at, updated_at, archived_at FROM accounts WHERE active",
);
query("SELECT account_id, tenant_id, created_at, updated_at, archived_at FROM accounts WHERE (");
query(
  "WITH recovered_accounts AS (SELECT account_id, tenant_id, created_at FROM accounts) SELECT account_id FROM recovered_accounts",
);
query(
  "WITH recovered_accounts AS (SELECT account_id, tenant_id, created_at FROM accounts) SELECT (",
);

// No CONFLICT clause: malformed INSERT/WITH must preserve legacy policy clears.
query("INSERT\n INTO items (id) VALUES (1)");
query("INSERT\n INTO items (id) SELECT (");
query("WITH v AS (INSERT INTO items (id) VALUES (1) RETURNING id) SELECT id FROM v");
query("WITH v AS (INSERT INTO items (id) SELECT (");
