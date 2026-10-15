import type { QueryOptions } from './unselected.mts';
// The global utility must preserve this unconfigured module's type identity.
export async function unselectedIndexed(ids: string[], run: NonNullable<QueryOptions['query']>) {
  for (const id of ids) await run('SELECT ' + id);
}
