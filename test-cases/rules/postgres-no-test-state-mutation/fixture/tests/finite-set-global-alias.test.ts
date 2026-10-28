import { query, sql } from '@example/db';
// A global constructor reference can be replaced before this guard runs.
const constructor = globalThis.Set;
export function guarded(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('unsafe');
  return query(sql`/* globalAlias */ SELECT 1 FROM `.append(table));
}
