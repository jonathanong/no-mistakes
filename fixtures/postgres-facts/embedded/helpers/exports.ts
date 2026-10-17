import sql from "sql-template-strings";
import type { SQLStatement } from "sql-template-strings";

export function plain(): string {
  return "SELECT 1";
}

export const arrow = (column: string) => `(${column} IS NULL)`;

export function repeated(left: string, right: string): string {
  return `${left} = ${right} AND ${left} IS NOT NULL`;
}

export function multiline(column: string, other: string): string {
  return `(${column} IS NULL
    OR ${other} IS NULL
    AND EXISTS (SELECT 1 FROM t))`;
}

export function withBlockBody(column: string): string {
  return `${column} IS NULL`;
}

export function withDefault(column: string = "x"): string {
  return `${column} IS NULL`;
}

export function withRest(...columns: string[]): string {
  return `${columns} IS NULL`;
}

export function withOptional(column?: string): string {
  return `${column} IS NULL`;
}

export function withDestructuring({ column }: { column: string }): string {
  return `${column} IS NULL`;
}

export function withExpression(column: string): string {
  return `${column.length} IS NULL`;
}

export function withOtherName(column: string): string {
  return `${other} IS NULL`;
}

export function withStatement(column: string): string {
  const copy = column;
  return `${copy} IS NULL`;
}

export async function asynchronous(column: string): Promise<string> {
  return `${column} IS NULL`;
}

export function withTag(column: string): string {
  return `${tag}${column}`;
}

export function* generated(column: string) {
  return `${column} IS NULL`;
}

export default function visibleByDefault(column: string): string {
  return `${column} IS NULL`;
}

export const expression = function (column: string) {
  return `${column} IS NOT NULL`;
};

export const blockArrow = (column: string) => {
  return `${column} IS NULL`;
};

// A function expression's own name would shadow the const inside its body.
export const mismatched = function other(column: string) {
  return `${column} IS NULL`;
};

export const asyncArrow = async (column: string) => `${column} IS NULL`;

export const noInterpolation = (column: string) => `IS NULL`;

let reassigned = (column: string) => `${column} IS NULL`;
reassigned = other;
export { reassigned };

export function returnsBuilder(query: SQLStatement): SQLStatement {
  return query;
}

export function appendsBuilder(query: SQLStatement, tier: string): SQLStatement {
  query.append(sql` AND tier = ${tier}`);
  return query;
}
