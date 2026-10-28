import { query, sql } from '@example/db';

const constructorAlias = Set;
export function constructorEscapedBeforeGuard(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('unsafe');
  return query(sql`/* escapedConstructorReference */ SELECT 1 FROM `.append(table));
}
