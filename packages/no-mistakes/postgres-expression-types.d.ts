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
  | { kind: "nullTest"; negated: boolean }
  | { kind: "distinctness"; negated: boolean }
  | { kind: "parameter"; placeholder: string }
  | { kind: "typedLiteral"; dataType: string; value: string; sql: string }
  | { kind: "literal"; sql: string; value: PostgresSqlLiteralValue }
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
  | "nullOperand"
  | "distinctLeft"
  | "distinctRight"
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
  | { kind: "nullTest"; negated: boolean }
  | { kind: "distinctness"; negated: boolean }
  | { kind: "parameter"; placeholder: string }
  | { kind: "typedLiteral"; dataType: string; value: string; sql: string }
  | { kind: "literal"; sql: string; value: PostgresSqlLiteralValue }
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

/** Numbers preserve decimal spelling; string values are parser-decoded. */
export type PostgresSqlLiteralValue =
  | { kind: "null" }
  | { kind: "string"; value: string }
  | { kind: "number"; value: string }
  | { kind: "boolean"; value: boolean }
  | { kind: "other"; sql: string };
