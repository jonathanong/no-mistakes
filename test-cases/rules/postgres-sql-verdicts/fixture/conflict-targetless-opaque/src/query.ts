import { query, sql } from "@example/db";
// The tail can still add ORDER BY, so this order finding stays an abstain.
query(sql`INSERT INTO items (id) SELECT id FROM items
 ON CONFLICT (id) DO NOTHING ${sql.raw(unknownSql)}`);
// A targetless multi-row arbiter is already in the prefix. The tail cannot
// insert a conflict target between ON CONFLICT and DO NOTHING.
query(sql`INSERT INTO items (id)
 SELECT input.id FROM unnest($1::uuid[]) AS input(id)
 ON CONFLICT DO NOTHING ${sql.raw(unknownSql)}`);
