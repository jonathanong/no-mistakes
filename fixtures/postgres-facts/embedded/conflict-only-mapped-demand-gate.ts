import { query, sql } from "@example/db";
// Escaped SQL newlines require explicit physical source positions.
query(
  "SELECT\n account_id, tenant_id, created_at, updated_at, archived_at FROM accounts WHERE active",
);
query("SELECT\n account_id, tenant_id, created_at, updated_at, archived_at FROM accounts WHERE (");
query(
  "WITH\n recovered_accounts AS (SELECT account_id, tenant_id, created_at FROM accounts) SELECT account_id FROM recovered_accounts",
);
query(
  "WITH\n recovered_accounts AS (SELECT account_id, tenant_id, created_at FROM accounts) SELECT (",
);
// A real INSERT hidden by its outer SELECT must retain its mapped conflict sites.
query(
  "WITH\n inserted AS (INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ON CONFLICT (id) DO NOTHING RETURNING id) SELECT id FROM inserted",
);
query(
  "SELECT\n account_id, tenant_id, created_at, updated_at, archived_at, 'INSERT CONFLICT' AS label FROM accounts",
);
query(
  "SELECT\n account_id, tenant_id, created_at, updated_at, archived_at FROM accounts /* INSERT CONFLICT */",
);
// Keep the existing opaque mapped prefix and its uncertainty handling.
query(sql`WITH\n recovered_accounts AS (
 SELECT account_id FROM accounts
) SELECT account_id FROM recovered_accounts ${sql.raw(unknownSql)}`);
// Ordinary INSERTs cannot have a conflict-policy finding, even when malformed.
query("INSERT\n INTO items (id) VALUES (1)");
query("INSERT\n INTO items (id) SELECT (");
// A mapped opaque INSERT prefix keeps uncertainty, not a partial policy proof.
query(sql`INSERT\n INTO items (id) SELECT id FROM items
 ON CONFLICT (id) DO NOTHING ${sql.raw(unknownSql)}`);
