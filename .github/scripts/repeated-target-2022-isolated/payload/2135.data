import type { PostgresSqlWrapper } from "./postgres-wrapper-types";
import type {
  PostgresSqlExpression,
  PostgresSqlIdentifier,
  PostgresSqlName,
  PostgresSqlType,
} from "./postgres-source-types";
export interface PostgresSqlFunctionArgument {
  name: PostgresSqlIdentifier | null;
  mode: string | null;
  dataType: PostgresSqlType;
  default: PostgresSqlExpression | null;
}
export interface PostgresSqlFunction {
  /** Non-executing declaration; opaque bodies have incomplete child facts. */
  wrapper: PostgresSqlWrapper;
  name: PostgresSqlName;
  arguments: PostgresSqlFunctionArgument[];
  returnType: PostgresSqlType | null;
  returnsSet: boolean;
  language: string | null;
  behavior: string | null;
  bodySql: string | null;
  orReplace: boolean;
  temporary: boolean;
  calledOnNull: string | null;
  parallel: string | null;
  configuration: string[];
  security: string | null;
}
