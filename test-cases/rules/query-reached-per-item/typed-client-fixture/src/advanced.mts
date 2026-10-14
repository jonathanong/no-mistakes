import type DefaultQuery from 'default-client';
import type { Namespace } from 'database-client';
import type * as WholeNamespace from 'database-client';
import type { Selected, Local, WholeNamespace as ReexportedNamespace } from './imported-types.mjs';
import type { Cyclic } from './cycle-a.mjs';
import { makeTools } from 'database-client';
import { createClient, createArray } from './providers.mjs';

export function defaults(ids: string[], execute: DefaultQuery) {
  for (const id of ids) execute(id);
}
export function nestedType(ids: string[], execute: Namespace.Query) {
  for (const id of ids) execute(id);
}
export function importedAlias(ids: string[], execute: Selected) {
  for (const id of ids) execute(id);
}
export async function nestedReceiver(ids: string[]) {
  const tools = (await makeTools()).tools;
  for (const id of ids) tools.query(id);
}
export function knownReceiver(ids: string[]) {
  const client = createClient();
  for (const id of ids) client.query(id);
  const array = createArray();
  for (const id of ids) array['0'](id); // A string key retains the static member-call projection.
}
// Unknown or cyclic annotations are retained as unselected identities, never guessed.
export function controls(ids: string[], a: Missing, b: WholeNamespace, c: Local, d: ReexportedNamespace, e: Cyclic) {
  for (const id of ids) { a(id); b(id); c(id); d(id); e(id); }
}
