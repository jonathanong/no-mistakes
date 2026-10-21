import { query, sql } from "@example/db";
// Opaque tails must retain their uncertainty when unused policy facts are omitted.
query(sql`SELECT\n id FROM items ${sql.raw(unknownSql)}`);
query(sql`INSERT\n INTO items (id) SELECT id FROM items
 ON CONFLICT (id) DO NOTHING ${sql.raw(unknownSql)}`);
