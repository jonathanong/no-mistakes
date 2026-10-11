import { query, sql } from "@example/db";
query(sql`${flag ? sql`
  -- no-mistakes-disable-next-line postgres-idempotent-insert
  INSERT INTO items (id) VALUES (1)` : sql`INSERT INTO items (id) VALUES (1) ON CONFLICT DO NOTHING`}`);
