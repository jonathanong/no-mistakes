import type { QueryOptions, TransactionQuery } from './database.mts';
import type * as Schema from './database.mts';
import type { Options } from './type-barrel.mts';
import { write as otherWrite } from './unselected.mts';
// No default object: the optional field's type owns this possible executor.
export async function projected(ids: string[], options: QueryOptions) {
  const query = options.query ?? otherWrite;
  for (const id of ids) await query(id);
}
export async function inline(ids: string[], options: { query?: TransactionQuery; count: number; method(): void; ['ignored']: string }) {
  const query = options.query ?? otherWrite;
  for (const id of ids) await query(id);
}
export async function namespace(ids: string[], query: NonNullable<Schema.QueryOptions['query']>) {
  for (const id of ids) await query(id);
}
export async function barrel(ids: string[], query: NonNullable<Options['query']>) {
  for (const id of ids) await query(id);
}
export async function annotatedAlternative(ids: string[], typed: NonNullable<QueryOptions['query']>, unknown) {
  const query = typed ?? unknown;
  for (const id of ids) await query(id);
}
