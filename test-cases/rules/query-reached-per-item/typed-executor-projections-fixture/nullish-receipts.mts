import { write } from './database.mts';
import type { QueryOptions, TransactionQuery } from './database.mts';
import { begin, client } from './receipt-factories.mts';
export async function returned(ids: string[], unknown) {
  const query = unknown ?? await begin();
  for (const id of ids) await query(id);
}
export async function bound(ids: string[], unknown) {
  const query = unknown ?? write.bind(null);
  for (const id of ids) await query(id);
}
export async function member(ids: string[], unknown) {
  const query = unknown ?? (await client()).query;
  for (const id of ids) await query(id);
}
// A configured type cannot turn a proven Promise into a callable executor.
async function typedPromise(ids: string[], typed: TransactionQuery, unknown) {
  const query = unknown ?? typed;
  for (const id of ids) await query(id);
}
async function indexedPromise(ids: string[], typed: NonNullable<QueryOptions['query']>, unknown) {
  const query = unknown ?? typed;
  for (const id of ids) await query(id);
}
export async function unawaited(ids: string[], unknown) {
  const query = unknown ?? begin();
  for (const id of ids) await query(id);
  await typedPromise(ids, begin() as unknown as TransactionQuery, unknown);
  await indexedPromise(ids, begin() as unknown as TransactionQuery, unknown);
}
