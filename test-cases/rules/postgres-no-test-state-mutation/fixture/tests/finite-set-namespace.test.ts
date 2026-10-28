import { query, sql } from '@example/db';
// Runtime TypeScript bindings cannot inherit the native constructor contract.
namespace Set { export const example = true; }
export function guarded(table: string) {
  const allowed = new Set(['safe_one', 'safe_two']);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query(sql`/* constructorBinding */ SELECT 1 FROM `.append(table));
}
