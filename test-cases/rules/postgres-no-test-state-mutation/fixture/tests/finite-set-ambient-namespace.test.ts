import { query, sql } from '@example/db';
// This ambient namespace is erased and leaves the native constructor accessible.
declare namespace Set { interface Example {} }
export function guarded(table: string) {
  const allowed = new Set(['safe_one', 'safe_two']);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query(sql`/* constructorBinding */ SELECT 1 FROM `.append(table));
}
