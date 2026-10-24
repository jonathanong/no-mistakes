import { query } from '@example/db';

export async function booleanKeyAndReturningBounds() {
  await query('SELECT 1 FROM boolean_keys WHERE enabled');
  // Both fixes are needed: the CTE target uses a bare boolean primary key.
  await query(`WITH u AS (
    UPDATE boolean_keys SET archived = false WHERE enabled RETURNING enabled
  ) UPDATE boolean_keys b SET archived = true FROM u WHERE b.enabled = u.enabled`);
  await query('SELECT 1 FROM boolean_keys WHERE archived');
}
