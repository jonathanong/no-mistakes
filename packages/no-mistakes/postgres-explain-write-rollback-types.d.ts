/** YAML options for the opt-in `postgres-explain-write-rollback` check. */
export interface PostgresExplainWriteRollbackOptions {
  /** Nonempty globs for standalone SQL scripts run with autocommit enabled. */
  sqlInclude: string[];
}
