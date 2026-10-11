import { query, sql } from "@example/db";
query(sql`SELECT ${flag ? sql`(SELECT id FROM accounts OFFSET 1),` : sql``} (SELECT id FROM accounts OFFSET 2)`);
