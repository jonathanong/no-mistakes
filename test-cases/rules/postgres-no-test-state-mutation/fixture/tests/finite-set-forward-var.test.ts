import { query, sql } from '@example/db';

function FakeCtor(_values: string[]) { return { has: (_value: string) => true }; }
export function forwardVarConstructor(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query(sql`/* forwardVarConstructor */ SELECT 1 FROM `.append(table));
}
// This binding is visible inside the earlier function after module initialization.
var Set = FakeCtor;
