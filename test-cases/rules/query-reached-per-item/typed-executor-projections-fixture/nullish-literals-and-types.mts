import { write } from './database.mts';
import type { QueryOptions } from './database.mts';
export async function literals(ids: string[]) {
 const query = null ?? write;
 for (const id of ids) await query(id);
}
export async function known(ids: string[], unknown) {
 const query = write ?? unknown;
 for (const id of ids) await query(id);
}
export async function bound(ids: string[], typed: NonNullable<QueryOptions['query']>, unknown) {
 const selected = typed.bind(null);
 const query = selected ?? unknown;
 for (const id of ids) await query(id);
}
// Recursive annotations and barrels cannot invent a configured leaf identity.
type A = { query: B['query'] };
type B = { query: A['query'] };
export async function circular(ids: string[], query: A['query']) {
 for (const id of ids) await query(id);
}
import type { Options as CyclicOptions } from './cycle-a.mts';
export async function circularBarrel(ids: string[], query: CyclicOptions['query']) {
 for (const id of ids) await query(id);
}
// Untyped and computed fields cannot borrow a typed sibling identity.
declare const dynamic: unique symbol;
type PartialShape = { query; [dynamic]: QueryOptions['query'] };
export async function unannotated(ids: string[], query: PartialShape['query']) {
 for (const id of ids) await query(id);
}
