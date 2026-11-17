import { type QueryExecutor } from "@example/db";
import { sql } from "@example/sql";

export function hiddenParameter(where: unknown, query: QueryExecutor) {
  return query(sql`SELECT * FROM t WHERE ${where}`);
}

export function hiddenAlias(where: unknown, query: QueryExecutor) {
  const clause = where;
  return query(sql`SELECT * FROM t WHERE ${clause}`);
}

export function hiddenReassigned(where: unknown, query: QueryExecutor) {
  let clause: unknown = 1;
  clause = where;
  return query(sql`SELECT * FROM t WHERE ${clause}`);
}

export function hiddenWrittenParameter(where: unknown, fragment: unknown, query: QueryExecutor) {
  where = fragment;
  return query(sql`SELECT * FROM t WHERE ${where}`);
}

export function hiddenDefaultFragment(query: QueryExecutor, where: unknown = sql`id = 1`) {
  return query(sql`SELECT * FROM t WHERE ${where}`);
}

export function visibleScalar(id: number, query: QueryExecutor) {
  return query(sql`SELECT * FROM t WHERE id = ${id}`);
}

export function visibleDefaultScalar(query: QueryExecutor, id: unknown = 1) {
  return query(sql`SELECT * FROM t WHERE id = ${id}`);
}

export function visibleLetScalar(query: QueryExecutor, id: number) {
  let scalar = id;
  return query(sql`SELECT * FROM t WHERE id = ${scalar}`);
}
