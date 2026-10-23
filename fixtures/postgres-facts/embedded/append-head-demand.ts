import sql from "sql-template-strings";
import { query, createPool, type DbClient } from "@example/db";
const q = sql`/* named */ SELECT ${id}`;
if (condition) q.append(sql` WHERE owner = ${owner}`);
for (const value of values) q.append(sql` AND n = ${value}`);
query(q);
const alias = query;
alias(sql`SELECT 2`.append(" LIMIT 1"));
const pool = createPool();
pool.query(q);
function typed(client: DbClient) { client.query(sql`SELECT 3`.append(" LIMIT 1")); }
// Diagnostic-only undeclared receivers must retain their literal builder head.
unknown.query(q);
