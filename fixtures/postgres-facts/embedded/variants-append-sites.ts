import { query, sql } from "@example/db";
const common = sql` WHERE id = 1`;
const q = sql`SELECT id FROM accounts`;
// Equivalent alternatives must retain both contributing append sites.
if (flag) q.append(common);
else q.append(common);
query(q);
query(sql`${q}`);
query(flag ? sql`SELECT id FROM accounts`.append(common) : sql`SELECT id FROM accounts`);
