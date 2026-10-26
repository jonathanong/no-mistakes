import { read } from "@example/db";
import type { QueryExecutor, QueryOptions } from "@example/executor-types";
declare const fallback: unknown;
declare const key: string;
export function wholeDefault({ query }: QueryOptions = fallback) {
  // CLEAR
  query("SELECT 1"); // ADVISE
}
// A callable-object signature must not hide its directly configured property.
export function callableObject({ query }: { (): void; query: QueryExecutor }) {
  query("SELECT 1"); // CLEAR
}
// A computed property is unsupported; keep its known default executor escape.
// ADVISE_NEXT
export function computed({ [key]: query = read }: QueryOptions) {
  query("SELECT 1"); // ADVISE: the unresolved computed key retains this bound call.
}
export function unrelatedRest(...values) {
  // CLEAR
  return values.length;
}
