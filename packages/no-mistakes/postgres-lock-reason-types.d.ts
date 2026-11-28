import type { Unanalyzable } from "./traversal-types";

/** YAML options for the opt-in `postgres-lock-reason` check. */
export interface PostgresLockReasonOptions {
  /** Rule-local source file include globs. */
  include?: string[];
  /** Rule-local source file exclude globs (for example test files and test helpers). */
  exclude?: string[];
  /** Native SQL file include globs. */
  sqlInclude?: string[];
  /** Module specifier exporting configured PostgreSQL executors. */
  importSpecifier?: string;
  /** Explicit executor function or method names. Without `importSpecifier`, `[]` selects no executor calls. With `importSpecifier`, `[]` keeps the default `[query, read, write]`. Factory and type receivers select executors independently. */
  executorNames?: string[];
  /** Additional executor names appended to the defaults. */
  extraExecutorNames?: string[];
  /** Executor names excluded from scanning. */
  excludeExecutorNames?: string[];
  /** Factory function names whose returned or callback values act as executors. */
  executorFactoryNames?: string[];
  /** TypeScript type names whose values act as executors. */
  executorTypeNames?: string[];
  /** Option property names on executor calls whose values select an executor override, such as `query` for `{ query: tx }`. */
  executorOptionProperties?: string[];
  /** Structured executor option properties whose values select an executor override, such as `{ name: "query" }` for `{ query: tx }`. */
  executorOptions?: { name: string }[];
  /** Trusted tagged template helpers whose interpolated SQL is scanned. */
  trustedSqlTags?: { module: string; name: string }[];
  /** Configured TypeScript or SQL helper function names that acquire locks. */
  helpers?: string[];
  /** Policy for unresolved SQL, parse failures, stored routine bodies, or unresolved executors (default: "advise"). */
  unanalyzable?: Unanalyzable;
  /** Report configured executorFactoryNames / executorTypeNames absent from scanned imports. */
  reportUnmatchedExecutorNames?: boolean;
}
