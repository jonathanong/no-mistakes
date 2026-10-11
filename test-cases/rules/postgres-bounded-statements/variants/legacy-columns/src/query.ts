import { query, sql } from "@example/db";
query(sql`SELECT a.id FROM accounts a CROSS JOIN accounts b`);
