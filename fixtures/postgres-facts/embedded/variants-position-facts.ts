import { query } from "@example/db";
export function run(flag: boolean) {
  query(sql`SELECT id FROM ${flag
    ? sql`accounts WHERE id = $1`
    : sql`accounts`}`);
  query(sql`SELECT ${flag
    ? sql`id`
    : sql`*`} FROM accounts`);
  query(sql`SELECT created_at AS ordered FROM accounts WHERE (${flag
    ? sql`id`
    : sql`created_at`}) = $1 ORDER BY (ordered)`);
  query(sql`UPDATE accounts SET ${flag
    ? sql`id = $1`
    : sql`created_at = $1`}`);
  query(flag
    ? "INSERT INTO accounts (id) VALUES (1)"
    : "INSERT INTO accounts (id) SELECT id FROM pending ON CONFLICT (id) DO NOTHING");
  query(flag
    ? "TRUNCATE accounts"
    : "SELECT 1");
  query(sql`SELECT id FROM accounts WHERE id = ANY($1) ${flag
    ? sql`ORDER BY id FOR UPDATE`
    : sql`FOR UPDATE`}`);
  query(sql`UPDATE accounts SET id = $1 RETURNING ${flag
    ? sql`id`
    : sql`*`}`);
  // Repeated writes must retain distinct physical column origins.
  query(sql`UPDATE accounts SET ${flag ? sql`id = $1` : sql`created_at = $1`}; UPDATE accounts SET created_at = $2`);
  query(flag
    ? "MERGE INTO accounts USING pending ON accounts.id = pending.id WHEN MATCHED THEN UPDATE SET (id, created_at) = (pending.id, pending.created_at) WHEN NOT MATCHED THEN INSERT (id, created_at) VALUES (pending.id, pending.created_at)"
    : "MERGE INTO accounts USING pending ON accounts.id = pending.id WHEN MATCHED THEN UPDATE SET id = pending.id WHEN NOT MATCHED THEN INSERT (id) VALUES (pending.id)");
  query(sql`UPDATE accounts SET id = $1 RETURNING *; UPDATE users SET id = $2 RETURNING ${flag ? sql`id` : sql`*`}`);
  query(sql`UPDATE accounts SET search_path = $1; SET ${flag ? sql`search_path = public` : sql`search_path = ''`}; SET search_path = public`);
  // Ordinary calls must keep their legacy projection and Debug representation.
  query("SELECT id FROM accounts WHERE id = $1");
}
