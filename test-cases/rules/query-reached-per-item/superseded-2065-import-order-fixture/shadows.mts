import type { NonNullable } from './unselected.mts';
// These authored bindings must not borrow the imported utility identity.
export async function generic<NonNullable>(ids: string[], query: NonNullable) {
  for (const id of ids) await query(id);
}
export namespace Local {
  type NonNullable<T> = (sql: string) => Promise<unknown>;
  export async function local(ids: string[], query: NonNullable<string>) {
    for (const id of ids) await query(id);
  }
}
