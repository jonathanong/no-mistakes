import sql from "sql-template-strings";
import type { SQLStatement } from "sql-template-strings";

// Lowered.
export function plain(id: string) {
  return sql`SELECT ${id}`;
}
export const arrowBlock = (id: string) => {
  return sql`SELECT ${id}`;
};
export const arrowExpression = (id: string) => sql`SELECT ${id}`;
export const functionExpression = function (id: string) {
  return sql`SELECT ${id}`;
};
export default function defaulted(id: string) {
  return sql`SELECT ${id}`;
}
export function callsOnly(id: string) {
  return plain(id);
}
export function forOf(items: string[]) {
  const q = sql`SELECT 1`;
  for (const item of items) q.append(item);
  return q;
}
export function nestedFunctionsAreOpaque() {
  const make = () => sql`SELECT 1`;
  return make();
}

// Not lowered: the parameter list or the declarations cannot be modeled.
export function destructuredParameter({ id }) {
  return sql`SELECT ${id}`;
}
export function restParameter(...ids: string[]) {
  return sql`SELECT ${ids}`;
}
export function duplicateParameter(id: string, id2: string) {
  const id = 1;
  return sql`SELECT ${id}`;
}
export function redeclaredInBlock(id: string) {
  {
    const inner = 1;
    const inner = 2;
  }
  return sql`SELECT ${id}`;
}
export function loopVariableShadowsParameter(item: string, items: string[]) {
  for (const item of items) {
    return sql`SELECT ${item}`;
  }
  return sql`SELECT 0`;
}
export async function asynchronous(id: string) {
  return sql`SELECT ${id}`;
}
export function* generator(id: string) {
  yield sql`SELECT ${id}`;
}
export function noSql(alias: string) {
  if (alias === "bad") throw new Error("bad");
}
export let reassigned = (id: string) => sql`SELECT ${id}`;
reassigned = (id: string) => sql`SELECT 0`;
export function loopWithPattern(items: string[]) {
  const q = sql`SELECT 1`;
  for (const [first] of items) q.append(first);
  for (var legacy of items) q.append(legacy);
  for (const item of items) {
    q.append(item);
    break;
  }
  return q;
}
export function withUninitialized() {
  let q;
  return sql`SELECT 1`;
}
export function ifElseIf(flag: boolean, other: boolean) {
  if (flag) return sql`SELECT 1`;
  else if (other) return sql`SELECT 2`;
  else return sql`SELECT 3`;
}
export function withBuilderTypes(a: SQLStatement, b: Array<SQLStatement>, c: string) {
  a.append(sql`x`);
}
