import type { NonNullable } from './unselected.mts';
import type { QueryOptions } from './database.mts';
// An imported utility with this spelling is not the global utility type.
export async function importedUtility(ids: string[], run: NonNullable<QueryOptions['query']>) {
  for (const id of ids) await run('SELECT ' + id);
}
