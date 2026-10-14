import type { TransactionQuery } from 'database-client';
import type { TransactionQuery as Other } from 'other-client';

function shared(ids: string[], execute) {
  for (const id of ids) execute(id);
}
// All callers share one graph; an unselected input removes a helper's effect proof.
export function selected(ids: string[], execute: TransactionQuery) { shared(ids, execute); }
export function unselected(ids: string[], execute: Other) { shared(ids, execute); }
export function members(ids: string[], execute: TransactionQuery) {
  for (const id of ids) execute.unselected(id);
}
