import type { Effort, Unanalyzable } from "./traversal-types";

export type PostgresLockWrittenRowsCheck =
  | "row-lock-unwritten-relation"
  | "table-lock"
  | "constant-advisory-lock"
  | "row-lock-outside-transaction";
export type PostgresLockWrittenRowsSeverity = "fail" | "advise" | "ignore";
/** At least one selector; all supplied selectors must match. */
export interface PostgresLockWrittenRowsAllow {
  relation?: string;
  /** Row predicate SQL value (such as `$1`), or advisory constants joined with comma-space. */
  key?: string;
  /** Exact root-relative path with slash separators. */
  file?: string;
  line?: number;
  reason: string;
}
export interface PostgresLockWrittenRowsWriteSet {
  file: string;
  transactionLine: number;
  relations: string[];
  reason?: string;
}
/** Configuration for the opt-in `postgres-lock-written-rows-only` check. */
export interface PostgresLockWrittenRowsOnlyOptions {
  include?: string[];
  exclude?: string[];
  importSpecifier?: string;
  executorNames?: string[];
  executorFactoryNames?: string[];
  executorTypeNames?: string[];
  trustedSqlTags?: { module: string; name: string }[];
  effort?: Effort;
  unanalyzable?: Unanalyzable;
  unanalyzableSql?: "fail" | "ignore" | "advise";
  checks?: Partial<Record<PostgresLockWrittenRowsCheck, PostgresLockWrittenRowsSeverity>>;
  allow?: PostgresLockWrittenRowsAllow[];
  writeSets?: PostgresLockWrittenRowsWriteSet[];
}
