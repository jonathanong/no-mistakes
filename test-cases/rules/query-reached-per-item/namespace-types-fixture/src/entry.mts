import type { Public } from './named.mjs';
import type { Local } from './local.mjs';
import type { Imported } from './namespace.mjs';
import type { Namespace as Unrelated } from './unrelated.mjs';
import type { Namespace as Shadow } from './shadow.mjs';
import type { Cyclic } from './cycle-a.mjs';
export function named(ids: string[], execute: Public.Query) {
  for (const id of ids) execute(id);
}
export function local(ids: string[], execute: Local.Query) {
  for (const id of ids) execute(id);
}
export function namespace(ids: string[], execute: Imported.Namespace.Query) {
  for (const id of ids) execute(id);
}
// Explicit roots must not guess local declarations, cyclic chains, or package prefixes.
export function controls(ids: string[], a: Unrelated.Query, b: Shadow.Query, c: Cyclic.Query) {
  for (const id of ids) { a(id); b(id); c(id); }
}
