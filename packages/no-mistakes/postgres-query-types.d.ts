import type { PostgresSqlQueryStatement } from "./postgres-query-statement-types";
export type * from "./postgres-query-statement-types";
import type {
  PostgresSqlIdentifier,
  PostgresSqlName,
  PostgresSqlSpan,
} from "./postgres-source-types";
/** Syntactic SELECT facts; IDs are deterministic and local to this query report. */
export interface PostgresSqlQuery {
  scopes: PostgresSqlQueryScope[];
  relations: PostgresSqlQueryRelation[];
  joins: PostgresSqlQueryJoin[];
  columns: PostgresSqlQueryColumn[];
  equalities: PostgresSqlQueryEquality[];
  exists: PostgresSqlQueryExists[];
  ctes: PostgresSqlQueryCte[];
  /** Data-modifying query bodies in original source order; SELECT CTEs are excluded. */
  nestedStatements: PostgresSqlQueryStatement[];
  unsupported: PostgresSqlQueryUnsupported[];
  complete: boolean;
}
export type PostgresSqlQueryClause =
  | "root"
  | "setBranch"
  | "cte"
  | "from"
  | "projection"
  | "where"
  | "joinOn"
  | "having"
  | "groupBy"
  | "orderBy"
  | "limit"
  | "offset"
  | "other";
export interface PostgresSqlQueryScope {
  id: number;
  parentScopeId: number | null;
  clause: PostgresSqlQueryClause;
  cteDefinitionId: number | null;
  setOperation: string | null;
  setQuantifier: string | null;
  span: PostgresSqlSpan | null;
}
export type PostgresSqlQueryRelationKind = "table" | "cte" | "derived" | "joined" | "unsupported";
export interface PostgresSqlQueryRelation {
  id: number;
  scopeId: number;
  kind: PostgresSqlQueryRelationKind;
  name: PostgresSqlName | null;
  alias: PostgresSqlIdentifier | null;
  columnAliases: PostgresSqlIdentifier[];
  cteId: number | null;
  subqueryScopeId: number | null;
  members: number[];
  lateral: boolean;
  span: PostgresSqlSpan | null;
}
export type PostgresSqlQueryJoinKind =
  | "inner"
  | "left"
  | "right"
  | "full"
  | "cross"
  | "semi"
  | "anti"
  | "other";
export interface PostgresSqlQueryJoin {
  id: number;
  scopeId: number;
  kind: PostgresSqlQueryJoinKind;
  left: number[];
  right: number[];
  constraint: string;
  usingColumns: PostgresSqlName[];
  span: PostgresSqlSpan | null;
}
export type PostgresSqlQueryColumnResolution = "resolved" | "unqualified" | "unknown" | "ambiguous";
export interface PostgresSqlQueryColumn {
  scopeId: number;
  clause: PostgresSqlQueryClause;
  name: PostgresSqlName;
  relationId: number | null;
  relationScopeId: number | null;
  resolution: PostgresSqlQueryColumnResolution;
  span: PostgresSqlSpan | null;
}
export interface PostgresSqlPredicateContext {
  mandatory: boolean;
  underOr: boolean;
  underNot: boolean;
  underCase: boolean;
  underBooleanTest: boolean;
  underOther: boolean;
}
export interface PostgresSqlQueryEquality {
  scopeId: number;
  clause: PostgresSqlQueryClause;
  joinId: number | null;
  left: PostgresSqlQueryColumn | null;
  right: PostgresSqlQueryColumn | null;
  context: PostgresSqlPredicateContext;
  span: PostgresSqlSpan | null;
}
export interface PostgresSqlQueryExists {
  scopeId: number;
  subqueryScopeId: number;
  /** The EXISTS node's own flag. Wrapping `NOT` is not folded into this value. */
  negated: boolean;
  /**
   * NOT operators that apply to this EXISTS: each wrapping `NOT` (parentheses
   * are transparent) plus one when `negated` is true.
   */
  notDepth: number;
  /** `notDepth % 2 === 1`. Distinct from `negated` and from `context.underNot`. */
  effectiveNegated: boolean;
  context: PostgresSqlPredicateContext;
  correlated: boolean;
  correlations: PostgresSqlQueryColumn[];
  span: PostgresSqlSpan | null;
}
export interface PostgresSqlQueryCte {
  id: number;
  ownerScopeId: number;
  queryScopeId: number;
  name: PostgresSqlIdentifier;
  columnAliases: PostgresSqlIdentifier[];
  recursive: boolean;
  referenced: boolean;
  used: boolean;
  cyclic: boolean;
  span: PostgresSqlSpan | null;
}
export interface PostgresSqlQueryUnsupported {
  scopeId: number;
  clause: PostgresSqlQueryClause;
  reason: string;
  span: PostgresSqlSpan | null;
}
