import { query, sql } from "@example/db";
query(sql`SELECT id FROM accounts OFFSET 1 ${flag ? sql`LIMIT 1` : sql`LIMIT 2`}`);
