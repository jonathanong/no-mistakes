import { query, sql } from '@example/db';

// A visible mutation anywhere in this source voids the builtin Set contract.
export function changeHas() {
  Set.prototype.has = () => true;
}
export function guardedAfterMutation(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query(sql`/* guardedAfterMutation */ SELECT 1 FROM `.append(table));
}
