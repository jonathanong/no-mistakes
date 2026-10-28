import { query, sql } from '@example/db';

// This local type name does not bind the runtime constructor.
type Set = { readonly unused: true };
export function typeOnlyConstructor(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query(sql`/* typeOnlyConstructor */ SELECT 1 FROM `.append(table));
}
