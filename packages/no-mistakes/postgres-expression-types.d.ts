import type {
  PostgresSqlIdentifier,
  PostgresSqlName,
  PostgresSqlSpan,
} from "./postgres-source-types";
/** Syntactic root; wrappers never promote a nested call through an unrelated operator. */
export type PostgresSqlExpressionRoot =
  | { kind: "columnReference"; name: PostgresSqlName }
  | {
      kind: "functionCall";
      name: PostgresSqlName;
      arguments: PostgresSqlCallArgument[];
      argumentsComplete: boolean;
      syntax: PostgresSqlFunctionSyntax;
      modifiers: string[];
    }
  | { kind: "parenthesized"; expression: PostgresSqlExpressionRoot }
  | { kind: "cast"; dataType: string; expression: PostgresSqlExpressionRoot }
  | { kind: "literal"; sql: string }
  | { kind: "binary"; operator: string }
  | { kind: "unary"; operator: string; expression: PostgresSqlExpressionRoot }
  | { kind: "case" | "subquery" | "other" };
export type PostgresSqlExpressionChildRole =
  | "argument"
  | "binaryLeft"
  | "binaryRight"
  | "caseOperand"
  | "caseWhenCondition"
  | "caseWhenResult"
  | "caseElse"
  | "castOperand"
  | "filterPredicate"
  | "parenthesizedExpression"
  | "unaryOperand"
  | "other";
export type PostgresSqlExpressionChildRoot =
  | { kind: "columnReference"; name: PostgresSqlName }
  | {
      kind: "functionCall";
      name: PostgresSqlName;
      argumentsComplete: boolean;
      syntax: PostgresSqlFunctionSyntax;
      modifiers: string[];
    }
  | { kind: "parenthesized" }
  | { kind: "cast"; castKind: string; dataType: string }
  | { kind: "literal"; sql: string }
  | { kind: "binary" | "unary"; operator: string }
  | { kind: "case" | "subquery" | "other" };
export interface PostgresSqlExpressionChild {
  role: PostgresSqlExpressionChildRole;
  index: number | null;
  argumentName?: PostgresSqlIdentifier;
  sql: string;
  span: PostgresSqlSpan | null;
  root: PostgresSqlExpressionChildRoot;
  children: PostgresSqlExpressionChild[];
  childrenComplete: boolean;
}
/** Bare CURRENT_TIMESTAMP is a value function; CURRENT_TIMESTAMP(3) uses call syntax. */
export type PostgresSqlFunctionSyntax = "call" | "value";
export interface PostgresSqlCallArgument {
  name: PostgresSqlIdentifier | null;
  sql: string;
  span: PostgresSqlSpan | null;
  root: PostgresSqlExpressionRoot;
}
