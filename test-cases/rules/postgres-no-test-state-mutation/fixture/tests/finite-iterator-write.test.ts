import { it } from 'vitest';
import { query } from '@example/db';

declare const replacement: (rows: string[]) => (name: string, test: (field: string) => void) => void;

// A method write invalidates the otherwise exact Vitest iterator identity.
it.each = replacement as typeof it.each;
it.each(['first'])('replaced iterator %s', async field => {
  const updates = { first: { sql: 'first_id = $2' } };
  const update = updates[field];
  await query(`UPDATE records SET ${update.sql} WHERE id = $1`);
});
