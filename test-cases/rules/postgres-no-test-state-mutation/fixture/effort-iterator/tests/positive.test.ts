import { it } from 'vitest';
import { query } from '@example/db';

declare const unknownId: string;
it.each(['safe', 'bad'] as const)('finite SQL fields', async field => {
  const unused = 'whatever';
  const updates = {
    safe: { sql: 'SELECT 1', value: unknownId },
    bad: { sql: 'DROP TABLE app.iterator_bad', value: unknownId }, // iterator DROP origin
  };
  const update = updates[field];
  await query(update.sql); // finite iterator call
});
