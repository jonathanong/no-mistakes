import { query, sql } from "@example/db";
const common = sql`SELECT pg_sleep(1)`;
const allowed = sql`SELECT 1`;
const own = sql``;
own.append(flag ? common : allowed);
// no-mistakes-disable-next-line postgres-sql-shape-policy
query(own);
const unrelated = sql``;
unrelated.append(otherFlag ? common : allowed);
