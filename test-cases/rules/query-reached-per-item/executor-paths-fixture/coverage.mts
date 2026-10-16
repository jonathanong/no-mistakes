import { write } from './database.mts'
import type { TransactionQuery, QueryOptions } from './database.mts'
import type { QueryOptions as BrokenOptions } from './broken.mts'
import { options, unrelatedFallback } from './entry.mts'

// Supplied options force both alternatives; defaults alone select the fallback.
export async function suppliedOptions(ids: string[], supplied: QueryOptions) {
  // Awaiting a plain value must preserve its property provenance too.
  await options(ids, await supplied)
  await unrelatedFallback(ids, supplied)
}
export async function typedCallback(ids: string[], run: TransactionQuery) {
  async function invoke(operation: (query: TransactionQuery) => Promise<void>) {
    await operation(run)
  }
  await invoke(async query => {
    for (const id of ids) await query('SELECT ' + id)
  })
}
export async function boundCallback(ids: string[], run: TransactionQuery) {
  async function invoke(operation: (query: TransactionQuery) => Promise<void>) {
    await operation(run.bind(null))
  }
  await invoke(async query => {
    for (const id of ids) await query('SELECT ' + id)
  })
}
async function local(sql: string) { return sql }
export async function definiteChoice(ids: string[]) {
  const query = write ?? local
  for (const id of ids) await query('SELECT ' + id)
}
export async function nullChoice(ids: string[]) {
  const query = null ?? write
  for (const id of ids) await query('SELECT ' + id)
}

// A type assertion cannot turn a known promise, object, or null into a callable.
async function promised() { return write }
export async function promiseCallback(ids: string[]) {
  async function invoke(operation: (query: TransactionQuery) => Promise<void>,
    run: TransactionQuery = promised() as unknown as TransactionQuery) {
    await operation(run)
  }
  await invoke(async query => {
    for (const id of ids) await query('SELECT ' + id)
  })
}
export async function objectCallback(ids: string[]) {
  async function invoke(operation: (query: TransactionQuery) => Promise<void>,
    run: TransactionQuery = {} as unknown as TransactionQuery) {
    await operation(run)
  }
  await invoke(async query => {
    for (const id of ids) await query('SELECT ' + id)
  })
}
export async function nullCallback(ids: string[]) {
  async function invoke(operation: (query: TransactionQuery) => Promise<void>,
    run: TransactionQuery = null as unknown as TransactionQuery) {
    await operation(run)
  }
  await invoke(async query => {
    for (const id of ids) await query('SELECT ' + id)
  })
}

// Unsupported keys, missing annotations, and malformed declarations prove no executor.
type Unsupported = { untyped; [key: string]: unknown }
type DynamicIndex = QueryOptions[keyof QueryOptions]
type NumericIndex = QueryOptions[1]
type SelfQuery = unknown
export async function unsupported(ids: string[], options: Unsupported,
  dynamic: DynamicIndex, numeric: NumericIndex, self: SelfQuery,
  broken: NonNullable<BrokenOptions['query']>, nonObject: TransactionQuery['query']) {
  for (const id of ids) {
    await options.untyped('SELECT ' + id)
    await dynamic('SELECT ' + id)
    await numeric('SELECT ' + id)
    await self('SELECT ' + id)
    await broken('SELECT ' + id)
    await nonObject('SELECT ' + id)
  }
}
