import { query, sql } from "@example/db";
const common = sql` WHERE pg_sleep(1) IS NULL`;
const allowed = sql` WHERE id = 1`;
// no-mistakes-disable-next-line postgres-sql-shape-policy
query(sql`SELECT id FROM accounts ${flag ? common : allowed}`);
const unrelated = sql`SELECT id FROM accounts`;
unrelated.append(otherFlag ? common : allowed);
