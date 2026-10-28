import { it } from 'vitest';
import { query } from '@example/db';

// Finite positive control: valueRead.
it.each(['first', 'second'])('valueRead %s', async field => {
  const map = { first: { sql: 'safe_one', value: 1 }, second: { sql: 'safe_two', value: 2 } };
  const selected = map[field];
  await query(`/* valueRead */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});
