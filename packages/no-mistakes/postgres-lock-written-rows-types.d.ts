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
/** A configured executor option property that selects the transaction client. */
export interface PostgresLockWrittenRowsExecutorOption {
  name: string;
}
/** Configuration for the opt-in `postgres-lock-written-rows-only` check. */
export interface PostgresLockWrittenRowsOnlyOptions {
  include?: string[];
  exclude?: string[];
  importSpecifier?: string;
  /** Replaces default executor names; mutually exclusive with extraExecutorNames. */
  executorNames?: string[];
  /** Extends query/read/write; mutually exclusive with executorNames. */
  extraExecutorNames?: string[];
  /** Removes names after replacement/defaults and extras are selected. */
  excludeExecutorNames?: string[];
  executorFactoryNames?: string[];
  executorTypeNames?: string[];
  /** Imported types that represent exclusively transaction executors. Empty by default. */
  transactionExecutorTypeNames?: string[];
  /** Static options properties overriding the transaction executor. Empty by default. */
  executorOptionProperties?: string[];
  /** Root-relative catalog for DDL relations; complete coverage enables trigger and non-null CTE key proofs. */
  schemaCatalogPath?: string;
  /** Adds routing property names to executorOptionProperties. Empty by default. */
  executorOptions?: PostgresLockWrittenRowsExecutorOption[];
  trustedSqlTags?: { module: string; name: string }[];
  effort?: Effort;
  unanalyzable?: Unanalyzable;
  unanalyzableSql?: "fail" | "ignore" | "advise";
  checks?: Partial<Record<PostgresLockWrittenRowsCheck, PostgresLockWrittenRowsSeverity>>;
  allow?: PostgresLockWrittenRowsAllow[];
  writeSets?: PostgresLockWrittenRowsWriteSet[];
}
