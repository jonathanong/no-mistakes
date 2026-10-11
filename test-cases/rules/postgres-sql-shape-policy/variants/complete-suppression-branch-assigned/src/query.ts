import { query, sql } from "@example/db";
const common = sql` WHERE pg_sleep(1) IS NULL`;
const allowed = sql` WHERE id = 1`;
let own = sql`SELECT id FROM accounts`;
if (flag) own = sql`SELECT id FROM accounts`;
else own = sql`SELECT id FROM projects`;
own.append(otherFlag ? common : allowed);
// no-mistakes-disable-next-line postgres-sql-shape-policy
query(own);
