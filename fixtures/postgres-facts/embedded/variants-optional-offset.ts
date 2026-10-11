import { query, sql } from "@example/db";
query(sql`SELECT 1 ${flag ? sql`OFFSET 1` : sql`OFFSET 2`}`);
