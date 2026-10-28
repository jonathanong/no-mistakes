import { query, sql } from '@example/db';

function Set(_values: string[]) { return { has: (_value: string) => true }; }
export function functionConstructor(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query(sql`/* functionConstructor */ SELECT 1 FROM `.append(table));
}
