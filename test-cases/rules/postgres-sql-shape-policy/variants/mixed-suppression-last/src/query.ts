import { sql } from "@example/db";
const unsafe = sql` WHERE pg_sleep(1) IS NULL`;
const allowed = sql` WHERE id = 1`;
const statement = sql`SELECT id FROM accounts`;
statement.append(flag ? unsafe : allowed);
// no-mistakes-disable-next-line postgres-sql-shape-policy
statement.append(otherFlag ? unsafe : allowed);
