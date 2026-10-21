import sql from "sql-template-strings";
import { read } from "@example/db";

// #2203 rewrite. An unknown selector keeps every arm. The throwing default adds none.
export function accessSql(access: string) {
  switch (access) {
    case "owned":
      return sql`SELECT id FROM orders WHERE account_id = $1`;
    case "member":
      return sql`SELECT id FROM orders WHERE member_id = $1`;
    case "all":
      return sql`SELECT id FROM orders WHERE account_id = $1 UNION ALL SELECT id FROM orders WHERE member_id = $2`;
    default:
      throw new Error("unknown access");
  }
}

export function byAccess(access: string) {
  return read(accessSql(access), [1]);
}

// #2198 rewrite. The orders arm omits account_id, so a judged call reports it.
export function tableName(kind: string) {
  switch (kind) {
    case "x":
      return "orders";
    case "y":
      return "accounts";
    default:
      throw new Error("unknown table");
  }
}

export function byTable(kind: string, id: string) {
  return read(`SELECT id FROM ${tableName(kind)} WHERE id = $1`, [id]);
}

// An inherited key is not an arm. Default throws, so this call publishes no SQL.
export function inherited(id: string) {
  return read(`SELECT id FROM ${tableName("constructor")} WHERE account_id = $1`, [id]);
}

// The name is not proof. The body returns, so the call stays unanalyzable.
function assertNever(value: string): string {
  return value;
}

export function namedOnly(kind: string) {
  switch (kind) {
    case "x":
      return sql`SELECT id FROM orders WHERE id = $1`;
    default:
      return assertNever(kind);
  }
}

export function notProof(kind: string) {
  return read(namedOnly(kind), [1]);
}

export function guardedLimit(limit: number) {
  if (!Number.isInteger(limit)) throw new Error("bad limit");
  return sql`SELECT id FROM orders WHERE member_id = $1 LIMIT ${limit}`;
}

export function limited(limit: number) {
  return read(guardedLimit(limit), [1]);
}

const START = "0";

export function nullishSql(after: string | null) {
  const afterId = after ?? START;
  return sql`SELECT id FROM orders WHERE id > ${afterId}`;
}

export function bounded(after: string | null) {
  return read(nullishSql(after));
}

// A reached unknown fragment stays unanalyzable. The fallback is not the only SQL.
export function nullishFragment(other: unknown) {
  return other ?? sql`SELECT id FROM orders WHERE account_id = $1`;
}

export function fragment(other: unknown) {
  return read(nullishFragment(other));
}

// A switch that returns statements is not an identifier clause.
export function statementHole(access: string) {
  return read(`SELECT ${accessSql(access)}`, [1]);
}

// A function that is not a SQL helper cannot fill an identifier hole.
function raw(kind: string) {
  return kind;
}

export function rawHole(kind: string, id: string) {
  return read(`SELECT id FROM ${raw(kind)} WHERE account_id = $1`, [id]);
}
