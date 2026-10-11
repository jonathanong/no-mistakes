import { query, sql } from "@example/db";
const common = sql` WHERE pg_sleep(1) IS NULL`;
const allowed = sql` WHERE id = 1`;
const own = sql`SELECT id FROM accounts`;
own.append(flag ? common : allowed);
// no-mistakes-disable-next-line postgres-sql-shape-policy
query(own);
