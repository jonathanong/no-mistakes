import { query, sql } from '@example/db';
// Runtime TypeScript bindings cannot inherit the native constructor contract.
import Set = require('../helpers/custom-set-constructor.cjs');
export function guarded(table: string) {
  const allowed = new Set(['safe_one', 'safe_two']);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query(sql`/* constructorBinding */ SELECT 1 FROM `.append(table));
}
