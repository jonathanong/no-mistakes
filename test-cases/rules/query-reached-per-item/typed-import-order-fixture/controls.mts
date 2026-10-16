import type { QueryOptions } from './database.mts';
// The builtin wrapper selects the unconfigured database type, not the custom import.
export async function builtin(ids: string[], query: NonNullable<QueryOptions['query']>) {
  for (const id of ids) await query(id);
}
export async function generic<NonNullable>(ids: string[], query: NonNullable) {
  for (const id of ids) await query(id);
}
