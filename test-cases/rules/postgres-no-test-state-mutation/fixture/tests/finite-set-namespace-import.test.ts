import * as Set from './opaque-constructor';
import { query, sql } from '@example/db';

export function runtimeNamespaceConstructor(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('unsafe');
  return query(sql`/* namespaceConstructor */ SELECT 1 FROM `.append(table));
}
