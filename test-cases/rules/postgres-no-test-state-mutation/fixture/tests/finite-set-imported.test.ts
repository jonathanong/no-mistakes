import { query, sql } from '@example/db';
import { Set } from '../helpers/custom-set';

export function importedConstructor(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query(sql`/* importedConstructor */ SELECT 1 FROM `.append(table));
}
