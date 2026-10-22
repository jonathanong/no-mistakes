import { query, sql } from "@example/db";
import { accountSql } from "./helper";

export function localVariants(flag: boolean, tenant: string, region: string) {
  const base = sql`SELECT id FROM accounts`;
  // Each branch misses a different column at the same physical relation token.
  query(sql`${base} ${flag ? sql`WHERE tenant_id = ${tenant}` : sql`WHERE region_id = ${region}`}`);
}

export function importedCalls() {
  query(accountSql());
  // Suppressing this execution must leave the unsuppressed helper finding intact.
  // no-mistakes-disable-next-line postgres-required-predicates
  query(accountSql());
}
