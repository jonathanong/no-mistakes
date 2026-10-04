import { query } from "@example/db";
query(`TABLE orders ORDER BY id LIMIT $1`);
// no-mistakes-disable-next-line postgres-sql-shape-policy
query(`TABLE orders ORDER BY id LIMIT $1`);
query(`TABLE orders ORDER BY id LIMIT 0`);
