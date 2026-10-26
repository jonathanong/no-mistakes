import type { QueryExecutor } from "@example/executor-types";
declare const fallback: unknown;
export function shorthand(query: QueryExecutor) {
  ({ query } = { query: fallback }); // CLEAR
  query("SELECT 1"); // ADVISE
}
// Erased TypeScript wrappers must not hide writes to the receiver binding.
export function asWrite(query: QueryExecutor) {
  (query as unknown) = fallback; // CLEAR
  query("SELECT 1"); // ADVISE
}
export function satisfiesWrite(query: QueryExecutor) {
  (query satisfies unknown) = fallback; // CLEAR
  query("SELECT 1"); // ADVISE
}
export function nonNullWrite(query: QueryExecutor) {
  query! = fallback; // CLEAR
  query("SELECT 1"); // ADVISE
}
export function assertionWrite(query: QueryExecutor) {
  (<unknown>query) = fallback; // CLEAR
  query("SELECT 1"); // ADVISE
}
// Unrelated numeric/private-field writes cannot invalidate the receiver.
export class UnrelatedWrites {
  #value = 0;
  run(query: QueryExecutor) {
    const seen = {};
    seen[0] = true; // CLEAR
    this.#value = 1; // CLEAR
    query("SELECT 1"); // CLEAR
  }
}
