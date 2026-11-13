import { sql } from "@example/db";
import type { SQLStatement } from "sql-template-strings";

export function known(kind: string) {
  switch (kind) {
    case "a":
      return sql`SELECT 1`;
    default:
      throw new Error("no");
  }
}

export function emptyDefault(kind: string) {
  switch (kind) {
    case "a":
      return sql`SELECT 1`;
    default:
  }
}

export function bareConst(kind: string) {
  switch (kind) {
    case "a":
      return sql`SELECT 1`;
    default: {
      const missing;
      throw new Error("no");
    }
  }
}

export function elseRedeclare(flag: boolean, name: string) {
  if (flag) {
    return sql`SELECT ${name}`;
  } else {
    const name = "x";
    return sql`SELECT ${name}`;
  }
}

export function forOfRedeclare(rows: string[]) {
  for (const row of rows) {
    const row = "x";
    return sql`SELECT ${row}`;
  }
  return sql`SELECT 1`;
}

export function nestedFunction(id: string) {
  function inner() {
    return 1;
  }
  return sql`SELECT ${id}`;
}

export function andLeft(flag: boolean) {
  const q = sql`SELECT 1`;
  if (q.append(...[flag]) && flag) q.append(sql` AND a = 1`);
  return q;
}

export function andRight(flag: boolean) {
  const q = sql`SELECT 1`;
  if (true && q.append(...[flag])) q.append(sql` AND a = 1`);
  return q;
}

export function nullishLeft() {
  const q = sql`SELECT 1`;
  const value = q.append(...[]) ?? "x";
  if (value) q.append(sql` AND a = 1`);
  return q;
}

export function nullishRight() {
  const q = sql`SELECT 1`;
  const value = null ?? q.append(...[]);
  if (value) q.append(sql` AND a = 1`);
  return q;
}

export function spreadUnknown(row: string) {
  const bag = { ...row, keep: "a" };
  return sql`SELECT 1`;
}

export function undefKey(flag: boolean) {
  const q = sql`SELECT 1`;
  if (undefined === flag) q.append(sql` AND a = 1`);
  return q;
}

export function eqAnd(a: boolean, b: boolean, flag: boolean) {
  const q = sql`SELECT 1`;
  if ((a && b) === flag) q.append(sql` AND a = 1`);
  return q;
}

export function openBag(row: SQLStatement) {
  const bag = { ...row, keep: "a" };
  return sql`SELECT 1`;
}

function mystery(): string {
  return "x";
}

export function defaultSql(kind: string) {
  switch (kind) {
    case "a":
      return "no";
    default:
      return mystery();
  }
}

export function boolDefault(flag: boolean) {
  switch (flag) {
    case "a":
      return "no";
    default:
      return mystery();
  }
}

function inner(name: string) {
  return name;
}

export function shortCall(flag: boolean) {
  const q = sql`SELECT 1`;
  if (flag) {
    q.append(sql` AND id = ${inner()}`);
  }
  return q;
}

function appender(statement: SQLStatement) {
  statement.append(sql` AND a = 1`);
  return sql`SELECT 2`;
}

export function notBuilder(name: string) {
  return appender(name);
}
