import { query, sql } from "@example/db";
const common = sql`SELECT pg_sleep(1)`;
const allowed = sql`SELECT 1`;
// no-mistakes-disable-next-line postgres-sql-shape-policy
query(flag ? common : allowed);
const unrelated = sql``;
unrelated.append(otherFlag ? common : allowed);
