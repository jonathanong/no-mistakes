import { it } from 'vitest';
import { query } from '@example/db';
declare function escape(value: unknown): void;
it.each(['safe', 'bad'] as const)('escaped map', async field => {
  const updates = { safe: 'SELECT 1', bad: 'DROP TABLE app.escaped_map' };
  escape(updates);
  await query(updates[field]); // residual escaped map
});
