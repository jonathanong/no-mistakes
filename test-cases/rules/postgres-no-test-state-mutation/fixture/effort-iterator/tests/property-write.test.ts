import { it } from 'vitest';
import { query } from '@example/db';
declare const replacement: string;
it.each(['safe', 'bad'] as const)('written map property', async field => {
  const updates = { safe: 'SELECT 1', bad: 'DROP TABLE app.written_map' };
  updates.safe = replacement;
  await query(updates[field]); // residual property write
});
