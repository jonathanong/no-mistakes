import sql from "sql-template-strings";
import { read } from "@example/db";
import { gate } from "./gate.mts";

declare const rest: unknown[];
declare const key: string;
declare const options: { a: { b: { c: string } } };

function start(id: string) {
  return sql`SELECT r.id FROM roots r WHERE r.owner = ${id} AND `;
}

// Reasonable arguments: the walk records a hole and the helper fills it.

export async function templateAndUndefined(id: string) {
  const query = start(id);
  query.append(gate("x", `y`, undefined));
  return read(query);
}

export async function nullAndNumber(id: string) {
  const query = start(id);
  query.append(gate("x", null, 5));
  return read(query);
}

export async function memberChain(id: string) {
  const query = start(id);
  query.append(gate("x", options.a.b.c));
  return read(query);
}

export async function objectAndArray(id: string) {
  const query = start(id);
  query.append(gate("x", { "quoted": 1, plain: id, flag: true }, [1, id, "s"]));
  return read(query);
}

// Arguments the walk cannot reason about leave the call unanalyzable.

export async function spreadInArray(id: string) {
  const query = start(id);
  query.append(gate("x", [...rest]));
  return read(query);
}

export async function holeInArray(id: string) {
  const query = start(id);
  query.append(gate("x", [, 1]));
  return read(query);
}

export async function spreadInObject(id: string) {
  const query = start(id);
  query.append(gate("x", { ...options }));
  return read(query);
}

export async function methodInObject(id: string) {
  const query = start(id);
  query.append(gate("x", { run() {} }));
  return read(query);
}

export async function computedKey(id: string) {
  const query = start(id);
  query.append(gate("x", { [key]: 1 }));
  return read(query);
}

export async function getterInObject(id: string) {
  const query = start(id);
  query.append(gate("x", { get value() { return 1; } }));
  return read(query);
}

export async function nestedObject(id: string) {
  const query = start(id);
  query.append(gate("x", { inner: { a: 1 } }));
  return read(query);
}

export async function builderInObject(id: string) {
  const query = start(id);
  const other = sql`SELECT 1`;
  query.append(gate("x", { other }));
  return read(query);
}

export async function builderInArray(id: string) {
  const query = start(id);
  const other = sql`SELECT 1`;
  query.append(gate("x", [other]));
  return read(query);
}

export async function callResult(id: string) {
  const query = start(id);
  query.append(gate("x", start(id)));
  return read(query);
}

export async function computedValue(id: string) {
  const query = start(id);
  query.append(gate("x", id + "y"));
  return read(query);
}

export async function substitutedTemplate(id: string) {
  const query = start(id);
  query.append(gate("x", `y${id}`));
  return read(query);
}

export async function spreadArgument(id: string) {
  const query = start(id);
  query.append(gate("x", ...rest));
  return read(query);
}
