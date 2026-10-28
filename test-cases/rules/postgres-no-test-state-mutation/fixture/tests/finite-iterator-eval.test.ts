import { it } from 'vitest';
import { query } from '@example/db';

it.each(['first'])('evaluated %s', async field => {
  const updates = { first: { sql: 'first_id = $2' } };
  const update = updates[field];
  // Direct eval can mutate the local record without a visible update reference.
  eval('update.sql = "other_id = $2"');
  await query(`/* evaluated */ UPDATE records SET ${update.sql} WHERE id = $1`);
});
