import type {
  PostgresSqlExpression,
  PostgresSqlIdentifier,
  PostgresSqlName,
} from "./postgres-source-types";

export interface PostgresSqlTriggerTransition {
  kind: string;
  name: PostgresSqlName;
}
/** UPDATE OF columns preserve source order and quoted identifiers. */
export type PostgresSqlTriggerEventKind = "insert" | "delete" | "truncate" | "update";
export interface PostgresSqlTriggerEvent {
  kind: PostgresSqlTriggerEventKind;
  updateOf: boolean;
  updateColumns: PostgresSqlIdentifier[];
}
export interface PostgresSqlTrigger {
  name: PostgresSqlName;
  table: PostgresSqlName;
  timing: string | null;
  events: string[];
  eventFacts: PostgresSqlTriggerEvent[];
  forEach: string | null;
  condition: PostgresSqlExpression | null;
  function: PostgresSqlName | null;
  arguments: string[];
  constraint: boolean;
  orReplace: boolean;
  referencedTable: PostgresSqlName | null;
  transitions: PostgresSqlTriggerTransition[];
  executionKind: string | null;
  characteristics: string | null;
}
