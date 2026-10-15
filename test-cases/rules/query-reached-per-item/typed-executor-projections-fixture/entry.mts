import { beginTransaction, write } from './database.mts';
import type { TransactionQuery, QueryOptions } from './database.mts';
export async function control(ids: string[], run: TransactionQuery) {
  for (const id of ids) await run('SELECT ' + id);
}
export async function indexed(ids: string[], run: NonNullable<QueryOptions['query']>) {
  for (const id of ids) await run('SELECT ' + id);
}
async function withTransaction(operation: (query: TransactionQuery) => Promise<void>) {
  const transaction = await beginTransaction();
  await operation(transaction);
}
export async function callback(ids: string[]) {
  await withTransaction(async query => {
    for (const id of ids) await query('SELECT ' + id);
  });
}
export async function options(ids: string[], options: QueryOptions = {}) {
  const query = options.query ?? write;
  for (const id of ids) await query('SELECT ' + id);
}
