import { query, sql } from "@example/db";
const safe = sql`/* deadlock-safe */ SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE`;
const unsafe = sql`SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE`;
query(flag ? safe : unsafe);
