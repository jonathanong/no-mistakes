/** A base table whose size is bounded by its application contract. */
export interface PostgresMaterializedViewBaseTableAllowance {
  table: string;
  /** Nonempty explanation of why this table does not need a query window. */
  reason: string;
}

/** An explicit exception to a materialized view shape finding. */
export interface PostgresMaterializedViewShapeAllowEntry {
  /** `materialized-view:<name>`. */
  object: string;
  reason: string;
}

/** Options for the `postgres-materialized-view-shape` catalog rule. */
export interface PostgresMaterializedViewShapeOptions {
  schemaCatalogPath: string;
  /** Number of indexes required in addition to the refresh index; default 1. */
  minAdditionalIndexes?: number;
  /** Functions accepted in predicates on indexed primary or unique keys, including equality. */
  windowFunctions?: string[];
  /** Tables exempted from the growing-table window requirement; default []. */
  allowBaseTables?: PostgresMaterializedViewBaseTableAllowance[];
  /** Explicit finding exceptions, each with a nonempty reason; default []. */
  allow?: PostgresMaterializedViewShapeAllowEntry[];
}
