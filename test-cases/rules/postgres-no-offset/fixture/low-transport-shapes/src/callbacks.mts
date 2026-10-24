import { read, beginTransaction, runWithTransaction } from "@example/db";
import type { QueryExecutor, QueryOptions, TransactionQuery } from "@example/executor-types";
declare const statement: string;

// The configured factory owns these callbacks; untyped receivers stay uncertain.
beginTransaction(async (query: TransactionQuery) => {
  query("SELECT 1"); // CLEAR
});
beginTransaction(function (query: TransactionQuery) {
  query("SELECT 1"); // CLEAR
});
beginTransaction(async (query) => {
  query(statement); // ADVISE
});
export function callbackWithOptions(options: QueryOptions) {
  return runWithTransaction(options.query, async (query: TransactionQuery) => {
    // CLEAR
    query("SELECT 1"); // CLEAR
  });
}
// Spread/object transport remains outside the direct callback argument contract.
// ADVISE_NEXT
runWithTransaction(...[read], async (query: TransactionQuery) => {
  query("SELECT 1"); // CLEAR
});
// ADVISE_NEXT
runWithTransaction({ query: read }, async (query: TransactionQuery) => {
  query("SELECT 1"); // CLEAR
});

declare const opaque: (value: unknown, callback: unknown) => unknown;
// A typed callback cannot prove how an unconfigured helper uses its other input.
export function opaqueCallback(query: QueryExecutor) {
  // ADVISE_NEXT
  opaque(query, async (transaction: TransactionQuery) => {
    transaction("SELECT 1"); // CLEAR
  });
}
