import { query, sql } from "@example/db";
const common = sql` WHERE pg_sleep(1) IS NULL`;
// no-mistakes-disable-next-line postgres-sql-shape-policy
query(flag ? sql`SELECT id FROM accounts`.append(common) : sql`SELECT id FROM accounts`);
