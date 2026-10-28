import { it } from 'vitest';
import { query } from '@example/db';
declare const replacement: string;
it.each(['safe', 'bad'] as const)('mutable callback parameter', async field => {
  field = replacement;
  const updates = { safe: 'SELECT 1', bad: 'DROP TABLE app.parameter_write' };
  await query(updates[field]); // residual parameter write
});
