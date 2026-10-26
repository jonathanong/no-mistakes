/** An intentional exception to `postgres-json-relation-ids`. */
export interface PostgresJsonRelationIdsAllowEntry {
  /** `table:<name>` or `column:<table>.<column>`. */
  object: string;
  /** Why this stored identifier does not need a same-row foreign key. */
  reason: string;
}

/** Options for the advisory `postgres-json-relation-ids` check rule. */
export interface PostgresJsonRelationIdsOptions {
  /** Required when `keyColumnMap` or `fingerprintNamePattern` is configured. */
  schemaCatalogPath?: string;
  /** Regex for top-level JSON object keys that look like row identifiers; unset disables this check. */
  idKeyPattern?: string;
  /** Maps matching JSON keys to candidate same-row foreign-key columns. */
  keyColumnMap?: Record<string, string>;
  /** Regex for unique `bytea` or `text` fingerprint column names; unset disables this check. */
  fingerprintNamePattern?: string;
  /** Explicit exceptions, each with a nonempty reason. */
  allow?: PostgresJsonRelationIdsAllowEntry[];
  include?: string[];
  exclude?: string[];
  sqlInclude?: string[];
  importSpecifier?: string;
  executorNames?: string[];
  executorFactoryNames?: string[];
  executorTypeNames?: string[];
  trustedSqlTags?: { module: string; name: string }[];
}
