import { query } from "@example/db";
query(sql`SELECT id FROM orders WHERE ${
  identifier // no-mistakes-disable-line postgres-sql-shape-policy
} NOT IN (SELECT id FROM archived)`);
