import { it } from 'vitest';
import { query } from '@example/db';

// Runtime SQL remains unknown: optionalEach.
it.each?.(['first', 'second'])('optionalEach %s', async field => {
  const map = { first: { sql: 'safe_one', value: 1 }, second: { sql: 'safe_two', value: 2 } };
  const selected = map[field];
  await query(`/* optionalEach */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});
