import type { NonNullable } from './unselected.mts';
import type { QueryOptions } from './database.mts';
// Type imports are hoisted; source position cannot turn this into the builtin.
export async function forward(ids: string[], query: NonNullable<QueryOptions['query']>) {
  for (const id of ids) await query(id);
}
