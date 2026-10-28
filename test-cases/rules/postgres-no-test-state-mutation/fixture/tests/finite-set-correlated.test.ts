import { query, sql } from '@example/db';
export function ordinary(table: string) {
  const allowed = new Set(['one', 'two', 'three', 'four', 'five', 'six']);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query(`/* correlatedOrdinary */ SELECT 1 FROM ${table} JOIN ${table} USING (id)`);
}
export function tagged(table: string) {
  const allowed = new Set(['one', 'two', 'three']);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query(sql`/* correlatedTagged */ SELECT 1 FROM `.append(table).append(sql` JOIN `).append(table).append(sql` USING (id)`));
}
