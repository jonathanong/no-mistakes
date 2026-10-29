import { it } from 'vitest';
import { query } from '@example/db';
it.each(['safe', '__proto__'] as const)('prototype selection', async field => {
  const updates = { safe: 'SELECT 1', bad: 'DROP TABLE app.unreachable' };
  await query(updates[field]); // residual prototype key
});
