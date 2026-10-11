import { query, sql } from "@example/db";
query(sql`SELECT id FROM accounts WHERE ${flag ? sql`
  id NOT IN (SELECT id FROM orders) -- findings: not-in-subquery
` : sql`id IN (SELECT id FROM orders)`}`);
query(sql`SELECT ${flag ? sql`
  COUNT(*) > 0 -- findings: count-for-existence
` : sql`EXISTS (SELECT 1 FROM accounts)`} FROM accounts`);
query(sql`SELECT a.id FROM accounts a WHERE ${flag ? sql`
  EXISTS (SELECT o.id FROM orders o WHERE o.id = a.id UNION SELECT o.id FROM orders o WHERE o.id = a.id) -- findings: correlated-exists-set-operation
` : sql`EXISTS (SELECT o.id FROM orders o WHERE o.id = a.id)`}`);
query(sql`SELECT id FROM accounts ${flag ? sql`
  LIMIT 2 -- findings: literal-limit
` : sql`LIMIT $1`}`);
query(sql`${flag ? sql`
  SELECT id FROM accounts WHERE id > $1 ORDER BY id LIMIT $2 -- findings: keyset-only-sweep
` : sql`SELECT id FROM accounts WHERE id > $1 AND active IS TRUE ORDER BY id LIMIT $2`}`);
query(sql`SELECT id FROM accounts WHERE ${flag ? sql`
  pg_sleep(1) IS NULL -- findings: banned-function-call
` : sql`id = $1`}`);
