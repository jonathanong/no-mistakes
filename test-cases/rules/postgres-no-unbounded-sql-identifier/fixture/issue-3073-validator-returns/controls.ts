import { type Query, assertWhitelistedSqlIdentifier } from "db";
import { assertWhitelistedSqlIdentifier as wrongValidator } from "./wrong.ts";
export function unchecked(query: Query, table: "alpha" | "beta") {
  return query(`SELECT * FROM ${table}`);
}
export function wrongModule(query: Query, value: string) {
  const table = wrongValidator(value);
  return query(`SELECT * FROM ${table}`);
}
export function shadowed(query: Query, value: string, assertWhitelistedSqlIdentifier: (value: string) => string) {
  const table = assertWhitelistedSqlIdentifier(value);
  return query(`SELECT * FROM ${table}`);
}
export function otherArgument(query: Query, value: string, other: string) {
  const table = assertWhitelistedSqlIdentifier(value, ["alpha"], other);
  return query(`SELECT * FROM ${other}`);
}
export function reassigned(query: Query, value: string) {
  const table = assertWhitelistedSqlIdentifier(value, ["alpha"], "table");
  value = process.env.UNCHECKED_TABLE;
  return query(`SELECT * FROM ${value}`);
}
export function capturedBeforeCall(query: Query, value: string) {
  const sql = `SELECT * FROM ${value}`;
  const table = assertWhitelistedSqlIdentifier(value, ["alpha"], "table");
  return query(sql);
}
export function boundOnly(query: Query, value: string) {
  const table = assertWhitelistedSqlIdentifier.bind(null, value, ["alpha"], "table");
  return query(`SELECT * FROM ${table}`);
}
export function callbackReturn(query: Query, value: string) {
  const table = [value].map(assertWhitelistedSqlIdentifier);
  return query(`SELECT * FROM ${table}`);
}
export function changedReturn(query: Query, value: string) {
  const table = assertWhitelistedSqlIdentifier(value, ["alpha"], "table").replace("alpha", process.env.UNCHECKED_TABLE);
  return query(`SELECT * FROM ${table}`);
}
export function laterDeclarator(query: Query, value: string) {
  const table = assertWhitelistedSqlIdentifier(value, ["alpha"], "table"), reset = (value = process.env.UNCHECKED_TABLE);
  return query(`SELECT * FROM ${value}`);
}
export function laterArgument(query: Query, value: string) {
  const table = assertWhitelistedSqlIdentifier(value, ["alpha"], value = process.env.UNCHECKED_TABLE);
  return query(`SELECT * FROM ${value}`);
}
