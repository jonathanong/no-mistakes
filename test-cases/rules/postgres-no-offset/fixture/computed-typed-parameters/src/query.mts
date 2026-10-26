import type { QueryOptions } from '@example/db';
import type { QueryOptions as OtherOptions } from '@unconfigured/db';
declare const key: string;
declare const dynamicSql: string;
declare const fallback: unknown;

// No runtime executor import can supply fallback tracking for these bindings.
export function computed({ [key]: query }: QueryOptions) {
  query('SELECT 1 OFFSET 9'); // UNKNOWN_LITERAL: syntax alone is not an executor proof.
  query(dynamicSql); // UNKNOWN_DYNAMIC
  query.query('SELECT 1'); // UNKNOWN_FUTURE: every use retains the negative binding.
}
export function nested({ [key]: { execute } }: { db: QueryOptions }) {
  execute('SELECT 1'); // UNKNOWN_NESTED
}
export function defaulted({ [key]: query = fallback }: QueryOptions) {
  query('SELECT 1'); // UNKNOWN_DEFAULT
}
export function direct({ query }: QueryOptions) {
  query('SELECT 1 OFFSET 2'); // DEFINITE_OFFSET: ordinary static members still prove SQL.
}
export function unconfigured({ [key]: query }: OtherOptions) {
  query('SELECT 1 OFFSET 3'); // CLEAR_UNCONFIGURED: type spelling is insufficient.
}
export function shadowed() {
  type QueryOptions = { arbitrary: unknown };
  function local({ [key]: query }: QueryOptions) {
    query('SELECT 1 OFFSET 4'); // CLEAR_SHADOW: the imported type has another identity.
  }
  return local;
}
