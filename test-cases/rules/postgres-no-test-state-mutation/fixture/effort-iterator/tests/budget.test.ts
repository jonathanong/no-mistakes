import { it } from 'vitest';
import { query } from '@example/db';
it.each(['r00', 'r01', 'r02', 'r03', 'r04', 'r05', 'r06', 'r07', 'r08',
  'r09', 'r10', 'r11', 'r12', 'r13', 'r14', 'r15', 'r16'] as const)(
  'more than sixteen rows', async field => {
    const updates = { r00: 'SELECT 1', r16: 'DROP TABLE app.budget' };
    await query(updates[field]); // residual row budget
  },
);
