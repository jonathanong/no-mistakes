import { query } from "@data-stores/psql";
query(sql`SELECT id FROM orders WHERE ${
  identifier // no-mistakes-disable-line postgres-sql-shape-policy
} NOT IN (SELECT id FROM archived)`);
