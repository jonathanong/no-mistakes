import { query, sql } from "@example/db";
query(sql`${flag ? sql`
  -- no-mistakes-disable-next-line postgres-require-query-annotation
  SELECT 1` : sql`/* items/read */ SELECT 1`}`);
