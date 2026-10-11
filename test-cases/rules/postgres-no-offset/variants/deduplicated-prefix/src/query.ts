import { query, sql } from "@example/db";
query(sql`SELECT ${flag ? sql`id` : sql`name, email`} FROM accounts OFFSET 1`);
