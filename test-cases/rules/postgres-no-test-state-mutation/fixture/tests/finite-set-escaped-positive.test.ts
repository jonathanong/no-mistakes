import { query, sql } from '@example/db';

export function escapedConstructor(table: string) {
  const allowed = new \u0053et(['safe_one', 'safe_two']);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query(sql`/* escapedConstructor */ SELECT 1 FROM `.append(table));
}
