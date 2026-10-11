import { query, sql } from "@example/db";
const common = sql`SELECT id FROM accounts OFFSET 7`;
query(sql`${flag ? common : sql`SELECT 1`}`);
// no-mistakes-disable-next-line postgres-no-offset
query(sql`${flag ? common : sql`SELECT 1`}`);
