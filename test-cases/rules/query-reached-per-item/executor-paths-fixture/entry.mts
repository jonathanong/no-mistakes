import { beginTransaction, write } from './database.mts'
import type { TransactionQuery, QueryOptions } from './database.mts'
import type { QueryOptions as OtherOptions } from './other.mts'
export async function control(ids: string[], run: TransactionQuery) {
  for (const id of ids) await run('SELECT ' + id)
}
export async function indexed(ids: string[], run: NonNullable<QueryOptions['query']>) {
  for (const id of ids) await run('SELECT ' + id)
}
async function withTransaction(operation: (query: TransactionQuery) => Promise<void>) {
  const transaction = await beginTransaction()
  await operation(transaction)
}
export async function callback(ids: string[]) {
  await withTransaction(async query => {
    for (const id of ids) await query('SELECT ' + id)
  })
}
export async function options(ids: string[], options: QueryOptions = {}) {
  const query = options.query ?? write
  for (const id of ids) await query('SELECT ' + id)
}
// Same spelling from an unselected module must not create executor provenance.
export async function unrelatedType(ids: string[], run: NonNullable<OtherOptions['query']>) {
  for (const id of ids) await run('SELECT ' + id)
}
async function local(sql: string) { return sql }
export async function unrelatedFallback(ids: string[], options: QueryOptions = {}) {
  const query = options.query ?? local
  for (const id of ids) await query('SELECT ' + id)
}
export async function unrelatedLocal(ids: string[]) {
  const query = local
  for (const id of ids) await query('SELECT ' + id)
}
// Indexed properties must reach declarations through sourced and renamed exports.
import type { QueryOptions as BarrelOptions, AliasOptions } from './barrel.mts'
export async function barrel(ids: string[], run: NonNullable<BarrelOptions['query']>) {
  for (const id of ids) await run('SELECT ' + id)
}
export async function renamed(ids: string[], run: NonNullable<AliasOptions['query']>) {
  for (const id of ids) await run('SELECT ' + id)
}
import './coverage.mts'
