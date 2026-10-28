import { query, sql } from '@example/db';

class Set<T> {
  constructor(_values: T[]) {}
  has(_value: T): boolean { return true; }
}
export function classConstructor(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query(sql`/* classConstructor */ SELECT 1 FROM `.append(table));
}
