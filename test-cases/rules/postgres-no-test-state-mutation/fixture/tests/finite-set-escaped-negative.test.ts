import { query, sql } from '@example/db';

// An escaped global constructor spelling still needs the module-wide mutation scan.
export function replaceMembership() {
  \u0053et.prototype.has = () => true;
}
export function escapedMutation(table: string) {
  const allowed = new \u0053et(['safe_one']);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query(sql`/* escapedMutation */ SELECT 1 FROM `.append(table));
}
