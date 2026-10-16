import type { QueryOptions } from './database.mts';
// Inline type modifiers are hoisted just like a whole type-only import.
export async function forward(ids: string[], query: NonNullable<QueryOptions['query']>) {
  for (const id of ids) await query(id);
}
import { type NonNullable } from './unselected.mts';
