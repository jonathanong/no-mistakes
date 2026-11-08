import { sql } from '@example/db';

export function candidates() {
  return sql`WITH shared_pool AS MATERIALIZED (
  SELECT id FROM orders
) SELECT id FROM shared_pool LIMIT 20`;
}
