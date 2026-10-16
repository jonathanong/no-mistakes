import type { QueryOptions } from './database.mts';
import type { TransactionQuery as OtherQuery } from './unselected.mts';
import { write as otherWrite } from './unselected.mts';
export async function outer(ids: string[]) {
  // This later local declaration owns the annotation for the whole outer scope.
  async function inner(query: NonNullable<QueryOptions['query']>) {
    for (const id of ids) await query(id);
  }
  type QueryOptions = { query: OtherQuery };
  await inner(otherWrite);
}
