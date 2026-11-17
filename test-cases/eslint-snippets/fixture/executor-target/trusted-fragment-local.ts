import { read, type QueryExecutor } from "@example/db";
import { sql } from "@example/sql";

function hiddenParameter(where, query: QueryExecutor) {
  return query(sql`SELECT * FROM t WHERE ${where}`);
}

function hiddenAlias(where, query: QueryExecutor) {
  const clause = where;
  return query(sql`SELECT * FROM t WHERE ${clause}`);
}

function hiddenReassigned(where, query: QueryExecutor) {
  let clause: unknown = 1;
  clause = where;
  return query(sql`SELECT * FROM t WHERE ${clause}`);
}

function hiddenWrittenParameter(where: unknown, fragment: unknown, query: QueryExecutor) {
  where = fragment;
  return query(sql`SELECT * FROM t WHERE ${where}`);
}

function hiddenDefaultFragment(query: QueryExecutor, where: unknown = sql`id = 1`) {
  return query(sql`SELECT * FROM t WHERE ${where}`);
}

function visibleScalar(id: number, query: QueryExecutor) {
  return query(sql`SELECT * FROM t WHERE id = ${id}`);
}

function visibleLiteral(query: QueryExecutor) {
  const id = 1;
  return query(sql`SELECT * FROM t WHERE id = ${id}`);
}

function visibleDefaultScalar(query: QueryExecutor, id: unknown = 1) {
  return query(sql`SELECT * FROM t WHERE id = ${id}`);
}

function visibleLetScalar(query: QueryExecutor, id: number) {
  let scalar = id;
  return query(sql`SELECT * FROM t WHERE id = ${scalar}`);
}

hiddenParameter(sql`id = 1`, read);
hiddenAlias(sql`id = 1`, read);
const predicate = sql`id = 1`;
hiddenParameter(predicate, read);
const typedPredicate: string = sql`id = 1`;
hiddenParameter(typedPredicate, read);
hiddenReassigned(sql`id = 1`, read);
hiddenWrittenParameter(1, sql`id = 1`, read);
hiddenDefaultFragment(read);
hiddenDefaultFragment(read, undefined);
hiddenDefaultFragment(read, true);
hiddenParameter(opaqueValue, read);
hiddenParameter(1, read);
hiddenAlias(2, read);
let scalarArg = 1;
hiddenParameter(scalarArg, read);
let mutableArg: unknown = 1;
mutableArg = sql`id = 1`;
hiddenParameter(mutableArg, read);
visibleScalar(1, read);
visibleLiteral(read);
visibleDefaultScalar(read);
visibleLetScalar(read, 1);

function callWithScalar(id: number) {
  hiddenParameter(id, read);
}
function callWithScalarUnion(id: number | string) {
  hiddenParameter(id, read);
}
function callWithNullableScalar(id: number | null | undefined) {
  hiddenParameter(id, read);
}
function callWithLiteralUnion(id: 1 | 2) {
  hiddenParameter(id, read);
}
callWithScalar(1);
callWithScalarUnion(1);
callWithNullableScalar(null);
callWithLiteralUnion(1);

function callWithMaybeUndefined(value: boolean | undefined) {
  hiddenDefaultFragment(read, value);
  visibleDefaultScalar(read, value);
}
callWithMaybeUndefined(undefined);
