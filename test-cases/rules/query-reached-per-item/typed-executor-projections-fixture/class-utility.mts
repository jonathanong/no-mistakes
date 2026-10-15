import type { QueryOptions } from './database.mts';
// A class occupies the type namespace too; its name is not the global utility.
class NonNullable<T> {}
export async function classUtility(ids: string[], run: NonNullable<QueryOptions['query']>) {
  for (const id of ids) await run('SELECT ' + id);
}
