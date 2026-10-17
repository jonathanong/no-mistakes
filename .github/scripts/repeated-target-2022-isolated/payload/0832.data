import type { PostgresSqlName } from "./postgres-source-types";
import type { PostgresSqlFunctionArgument } from "./postgres-function-types";

export interface PostgresSqlComment {
  objectType: string;
  name: PostgresSqlName;
  /** null means no signature was written; [] preserves an explicit (). */
  arguments: PostgresSqlFunctionArgument[] | null;
  comment: string | null;
}
export interface PostgresSqlAlterIndex {
  name: PostgresSqlName;
  ifExists: boolean;
  operation: PostgresSqlAlterIndexOperation;
}
export type PostgresSqlAlterIndexOperation =
  | { kind: "attachPartition"; partition: PostgresSqlName }
  | { kind: "rename"; name: PostgresSqlName };
