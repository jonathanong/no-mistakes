import { query, sql } from '@example/db';
declare enum Set { Safe = 'safe_one' }

export function ambientEnumLeavesRuntimeConstructor(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('unsafe');
  return query(sql`/* ambientEnum */ SELECT 1 FROM `.append(table));
}
