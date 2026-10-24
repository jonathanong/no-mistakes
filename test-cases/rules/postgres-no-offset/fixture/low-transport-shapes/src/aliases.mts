import type { QueryOptions } from "@example/executor-types";
declare const statement: string;
// Unsupported projections retain the original receiver escape, not a false proof.
export function nonQuery(options: QueryOptions) {
  const { other } = options; // ADVISE
  return other;
}
export function nested(options: QueryOptions) {
  const {
    query: { run },
  } = options; // ADVISE
  run(statement);
}
