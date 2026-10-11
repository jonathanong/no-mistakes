import { query, sql } from "@example/db";
query(sql`SELECT (SELECT id FROM accounts OFFSET 1), (SELECT id FROM accounts OFFSET 2) ${flag ? sql`` : sql`LIMIT 1`}`);
