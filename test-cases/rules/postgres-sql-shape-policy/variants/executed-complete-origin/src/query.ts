import { query, sql } from "@example/db";
const first = sql`SELECT pg_sleep(1)`;
const second = sql`SELECT pg_sleep(1)`;
const gate = runtimeFlag;
const own = sql``;
own.append(gate ? first : second);
// no-mistakes-disable-next-line postgres-sql-shape-policy
query(gate ? own : sql`SELECT 1`);
