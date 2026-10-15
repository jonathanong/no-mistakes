import type { QueryOptions } from './database.mts';
// Type bindings own their whole lexical scope, even before the declaration.
export async function forward(ids: string[], query: NonNullable<QueryOptions['query']>) {
  for (const id of ids) await query(id);
}
type NonNullable<T> = (sql: string) => Promise<unknown>;
