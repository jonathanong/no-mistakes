import { it } from 'vitest';
import { beginTransaction } from '../db/index';

declare function opaque(value: unknown): unknown;
it.each(['safe', 'bad'] as const)('map escapes through a bind', async field => {
  const tx = await beginTransaction();
  const map = {
    safe: { sql: 'SELECT 1' },
    bad: { sql: 'DROP TABLE app.escaped_map' },
  }[field];
  // The second argument exposes the local map to an opaque call.
  await tx(`${map.sql}`, [opaque(map)]);
});
