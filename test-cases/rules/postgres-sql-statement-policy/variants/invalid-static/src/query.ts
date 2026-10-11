import { query, sql } from "@example/db";
query(sql`${flag ? sql`-- parse failure stays a statement diagnostic
  SELECT FROM
` : sql`SELECT 1`}`);
