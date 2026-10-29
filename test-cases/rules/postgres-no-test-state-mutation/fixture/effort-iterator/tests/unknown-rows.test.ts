import { it } from 'vitest';
import { query } from '@example/db';
declare const fields: string[];
it.each(fields)('unknown rows', async field => {
  const updates = { safe: 'SELECT 1', bad: 'DROP TABLE app.unknown_rows' };
  await query(updates[field]); // residual unknown rows
});
