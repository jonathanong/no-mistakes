import { query } from '@example/db';

export async function candidates() {
  await query(`WITH candidates AS MATERIALIZED (
  SELECT id FROM orders
)
SELECT id FROM candidates LIMIT 20`);
  await query(`WITH candidates AS MATERIALIZED (SELECT id FROM orders LIMIT 200)
SELECT id FROM candidates LIMIT 20`);
  await query(`SELECT id FROM (SELECT id FROM orders) candidates LIMIT 20`);
}
