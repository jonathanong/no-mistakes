import type { TransactionQuery, QueryOptions } from './database.mts';
import type { TransactionQuery as OtherQuery, QueryOptions as OtherOptions } from './unselected.mts';
import { write as otherWrite } from './unselected.mts';
// This local utility intentionally shadows the global NonNullable utility.
type NonNullable<T> = (sql: string) => Promise<unknown>;
export async function shadowedUtility(ids: string[], run: NonNullable<QueryOptions['query']>) {
  for (const id of ids) await run('SELECT ' + id);
}
export async function genericShadow<TransactionQuery>(ids: string[], run: TransactionQuery) {
  for (const id of ids) await run('SELECT ' + id);
}
export async function unselectedType(ids: string[], run: OtherQuery) {
  for (const id of ids) await run('SELECT ' + id);
}
export async function unselectedMember(ids: string[], options: OtherOptions = {}) {
  const query = options.query ?? otherWrite;
  for (const id of ids) await query('SELECT ' + id);
}
async function withOtherTransaction(operation: (query: OtherQuery) => Promise<void>) {
  await operation(async sql => sql);
}
export async function unselectedCallback(ids: string[]) {
  await withOtherTransaction(async query => {
    for (const id of ids) await query('SELECT ' + id);
  });
}
export async function ordinary(ids: string[]) {
  const query = (sql: string) => sql;
  for (const id of ids) query('SELECT ' + id);
}

async function withSelectedTransaction(operation: (query: TransactionQuery) => Promise<void>) {
  await operation(async sql => sql);
}
// One configured context cannot override an unconfigured authored context.
export async function mixedContexts(ids: string[]) {
  async function shared(query) {
    for (const id of ids) await query('SELECT ' + id);
  }
  await withSelectedTransaction(shared);
  await withOtherTransaction(shared);
}
