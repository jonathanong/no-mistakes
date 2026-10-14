import type { TransactionQuery as Executor } from 'database-client';
import type * as Database from 'database-client';
import type { TransactionQuery as Unselected } from 'other-client';
import { writePool, advisoryLockPool, unrelatedPool } from 'database-client';

export async function first(ids: string[], query: Executor) {
  for (const id of ids) await query('SELECT $1', [id]);
}
export async function second(ids: string[], transaction: Database.TransactionQuery) {
  const execute = transaction;
  for (const id of ids) await execute('SELECT $1', [id]);
}
export async function third(ids: string[], txQuery: Executor) {
  for (const id of ids) await txQuery('SELECT $1', [id]);
}
export async function pools(ids: string[]) {
  const pool = writePool;
  const client = await pool.connect();
  const alias = client;
  for (const id of ids) await alias.query('SELECT $1', [id]);
  const lockClient = await advisoryLockPool.connect();
  for (const id of ids) await lockClient.query('SELECT $1', [id]);
  client.release();
}
// Equal local spellings must not create configured binding provenance.
export async function controls(ids: string[], query: Unselected) {
  const client = { query(id: string) { return id; } };
  for (const id of ids) client.query(id);
  for (const id of ids) await query('SELECT $1', [id]);
  const other = await unrelatedPool.connect();
  for (const id of ids) await other.query('SELECT $1', [id]);
}
export function shadowed<Executor>(ids: string[], query: Executor) {
  for (const id of ids) query(id);
}
