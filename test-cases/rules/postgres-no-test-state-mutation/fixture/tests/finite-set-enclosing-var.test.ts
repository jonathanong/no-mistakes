import { query, sql } from '@example/db';

export function enclosingVarConstructor(table: string) {
  function execute() {
    const allowed = new Set(['safe_one']);
    if (!allowed.has(table)) throw new Error('invalid table');
    return query(sql`/* enclosingVarConstructor */ SELECT 1 FROM `.append(table));
  }
  var Set = class { constructor(_values: string[]) {} has(_value: string) { return true; } };
  return execute();
}
