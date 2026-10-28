import Set from './opaque-constructor';
import { query, sql } from '@example/db';
// Imported constructors do not carry the native membership contract.
export function guarded(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('unsafe');
  return query(sql`/* defaultConstructor */ SELECT 1 FROM `.append(table));
}
