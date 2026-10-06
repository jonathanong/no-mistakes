import { query, sql } from '@example/db';
// no-mistakes-disable-next-line postgres-sql-statement-policy
query(sql`ALTER TABLE orders ADD COLUMN next_line text`);
query(sql`TRUNCATE orders`); // no-mistakes-disable-line postgres-sql-statement-policy
