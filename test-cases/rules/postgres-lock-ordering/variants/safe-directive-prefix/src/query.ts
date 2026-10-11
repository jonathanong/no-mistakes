import { query, sql } from "@example/db";
const prefix = sql`/* deadlock-safe */ SELECT id FROM accounts WHERE id IN ($1, $2)`;
query(sql`${prefix} ${flag ? sql`FOR UPDATE` : sql`FOR NO KEY UPDATE`}`);
