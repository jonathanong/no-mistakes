import { it } from 'vitest';
import { query } from '@example/db';

declare function consume(value: unknown): void;

// An unknown consumer can replace a method on the imported test function.
consume(it);
it.each(['first'])('escaped iterator %s', async field => {
  const updates = { first: { sql: 'first_id = $2' } };
  const update = updates[field];
  await query(`UPDATE records SET ${update.sql} WHERE id = $1`);
});
