import type { QueryOptions } from './database.mts';
type Options = { query: NonNullable<QueryOptions['query']> };
export async function indexed(ids: string[], query: Options['query']) {
  for (const id of ids) await query(id);
}
export async function context(callback: (query: NonNullable<QueryOptions['query']>) => Promise<void>) { await callback(async sql => {}); }
export async function contextual(ids: string[]) {
  await context(async query => {
    for (const id of ids) await query(id);
  });
}
// The utility import owns both nested annotations, despite its source position.
import type { NonNullable } from './unselected.mts';
