import { query, sql } from "@example/db";
const common = sql` WHERE pg_sleep(1) IS NULL`;
const own = sql`SELECT id FROM accounts`;
// Both branches share physical SQL, but each append belongs to the execution.
if (flag) own.append(common);
else own.append(common);
// no-mistakes-disable-next-line postgres-sql-shape-policy
query(own);
