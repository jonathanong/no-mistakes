import type { TransactionQuery } from './database.mts';
// Recursive object shape is productive; each authored next selects one field.
type Recursive = { next: Recursive; query: TransactionQuery };
export function finite(query: Recursive['next']['next']['query']) {
  for (const item of ['one']) query(item);
}
// This alias revisits an object through a different, finite field descriptor.
type Indexed = { query: Indexed['box']['query']; box: { query: TransactionQuery } };
export function finiteAlias(query: Indexed['query']) {
  for (const item of ['one']) query(item);
}
// Normalizing this descriptor recurs before producing an object or terminal.
type Growing = { query: Growing['query']['query'] };
export function growing(query: Growing['query']) {
  for (const item of ['one']) query(item);
}
type MutualA = { query: MutualB['query']['query'] };
type MutualB = { query: MutualA['query']['query'] };
export function mutualGrowing(query: MutualA['query']) {
  for (const item of ['one']) query(item);
}
