import type { Effort, Unanalyzable } from "./traversal-types";

/** Configuration for the `postgres-no-test-state-mutation` check rule. */
export interface PostgresNoTestStateMutationOptions {
  /** Explicit test entry globs. The rule has no inferred test convention. */
  testGlobs: string[];
  /** Explicit helper module globs included alongside test entries. */
  helperRoots?: string[];
  importSpecifier?: string;
  executorNames?: string[];
  executorFactoryNames?: string[];
  executorTypeNames?: string[];
  trustedSqlTags?: { module: string; name: string }[];
  /** `low` checks direct SQL syntax and one local const; `max` is the default. */
  effort?: Effort;
  unanalyzable?: Unanalyzable;
  /** Legacy spelling; `fail` is accepted as `report`. */
  unanalyzableSql?: "fail" | "ignore" | "advise";
}
