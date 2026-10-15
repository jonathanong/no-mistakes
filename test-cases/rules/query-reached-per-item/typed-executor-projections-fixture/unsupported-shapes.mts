import type { QueryOptions, TransactionQuery } from './database.mts';
// Empty or unrelated annotations cannot become configured executor evidence.
type Empty = {};
export async function empty(ids: string[], query: Empty['query']) {
  for (const id of ids) await query(id);
}
export async function numeric(ids: string[], query: QueryOptions[0]) {
  for (const id of ids) await query(id);
}
export async function dynamic(ids: string[], query: QueryOptions[keyof QueryOptions]) {
  for (const id of ids) await query(id);
}
export async function missing(ids: string[], query: QueryOptions['missing']) {
  for (const id of ids) await query(id);
}
export async function primitiveCallback(ids: string[], query: (value: number) => void) {
  for (const id of ids) query(id.length);
}
export async function genericCallback(ids: string[], query: <T>(value: T) => T) {
  for (const id of ids) query(id);
}
export async function functionArgument(ids: string[], query: (unknown, typed: TransactionQuery) => void) {
  for (const id of ids) query(id);
}
