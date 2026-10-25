import type { QueryOptions } from "@example/executor-types";
declare const statement: string;
// Data/receiver transfers do not execute SQL; unsupported projected calls stay unknown.
export function nonQuery(options: QueryOptions) {
  const { other } = options; // NON_SQL
  return other;
}
export function nested(options: QueryOptions) {
  const {
    query: { run },
  } = options; // NON_SQL
  run(statement); // ADVISE
}
