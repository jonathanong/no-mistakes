import { query, sql } from "@example/db";
query(sql`${flag ? sql`
  /**/ SELECT 1 -- findings: annotation
` : sql`/* items/read */ SELECT 1`}`);
query(sql`${flag ? sql`
  BEGIN
` : sql`COMMIT`}`);
query(sql`${flag ? sql`
  /* transaction */ ROLLBACK
` : sql`ROLLBACK TO SAVEPOINT s`}`);
