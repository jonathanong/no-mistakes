import { query } from "@example/db";
import { sql as rawSql } from "@example/raw-sql";

const openRecord = { vote: "users", tag: "posts" };
function getOpen(name: string) {
  if (!Object.hasOwn(openRecord, name)) throw new Error("no");
  return openRecord[name as keyof typeof openRecord];
}

const quiet = Object.freeze({ vote: "users", tag: "posts" } as const);
function getQuiet(name: string) {
  if (!Object.hasOwn(quiet, name)) return "users";
  return quiet[name as keyof typeof quiet];
}

const extra = Object.freeze({ vote: "users", tag: "posts" } as const);
function getExtra(name: string) {
  if (!Object.hasOwn(extra, name)) throw new Error("no");
  const value = extra[name as keyof typeof extra];
  return value;
}

const mutable = Object.freeze({ vote: "users", tag: "posts" } as const);
mutable.vote = "accounts";

const indexed = Object.freeze({ vote: "users", tag: "posts" } as const);
function writeIndexed(key: string) {
  indexed[key] = "accounts";
}
writeIndexed("vote");
function getIndexed(name: string) {
  if (!Object.hasOwn(indexed, name)) throw new Error("no");
  return indexed[name as keyof typeof indexed];
}
function getMutable(name: string) {
  if (!Object.hasOwn(mutable, name)) throw new Error("no");
  return mutable[name as keyof typeof mutable];
}

const escaped = Object.freeze({ vote: "users", tag: "posts" } as const);
function leak(value: object) {
  return value;
}
leak(escaped);
function getEscaped(name: string) {
  if (!Object.hasOwn(escaped, name)) throw new Error("no");
  return escaped[name as keyof typeof escaped];
}

const exportedTables = Object.freeze({ vote: "users", tag: "posts" } as const);
export { exportedTables };
function getExported(name: string) {
  if (!Object.hasOwn(exportedTables, name)) throw new Error("no");
  return exportedTables[name as keyof typeof exportedTables];
}

const spread = Object.freeze({ vote: "users", ...{ tag: "posts" } } as const);
function getSpread(name: string) {
  if (!Object.hasOwn(spread, name)) throw new Error("no");
  return spread[name as keyof typeof spread];
}

const mixed = Object.freeze({ vote: "users", tag: 1 } as const);
function getMixed(name: string) {
  if (!Object.hasOwn(mixed, name)) throw new Error("no");
  return mixed[name as keyof typeof mixed];
}

const shadowed = Object.freeze({ vote: "users", tag: "posts" } as const);
function getShadowed(shadowed: string) {
  if (!Object.hasOwn(shadowed, shadowed)) throw new Error("no");
  return shadowed[shadowed as keyof typeof shadowed];
}

const tooMany = Object.freeze({
  a: "users",
  b: "users",
  c: "users",
  d: "users",
  e: "users",
  f: "users",
  g: "users",
  h: "users",
  i: "users",
  j: "users",
  k: "users",
  l: "users",
  m: "users",
  n: "users",
  o: "users",
  p: "users",
  q: "users",
} as const);
function getTooMany(name: string) {
  if (!Object.hasOwn(tooMany, name)) throw new Error("no");
  return tooMany[name as keyof typeof tooMany];
}

export function run(kind: string) {
  query(rawSql`/* open */ DELETE FROM ${getOpen(kind)} WHERE id = $1`);
  query(rawSql`/* quiet */ DELETE FROM ${getQuiet(kind)} WHERE id = $1`);
  query(rawSql`/* extra */ DELETE FROM ${getExtra(kind)} WHERE id = $1`);
  query(rawSql`/* mutable */ DELETE FROM ${getMutable(kind)} WHERE id = $1`);
  query(rawSql`/* indexed */ DELETE FROM ${getIndexed(kind)} WHERE id = $1`);
  query(rawSql`/* escaped */ DELETE FROM ${getEscaped(kind)} WHERE id = $1`);
  query(rawSql`/* exported */ DELETE FROM ${getExported(kind)} WHERE id = $1`);
  query(rawSql`/* spread */ DELETE FROM ${getSpread(kind)} WHERE id = $1`);
  query(rawSql`/* mixed */ DELETE FROM ${getMixed(kind)} WHERE id = $1`);
  query(rawSql`/* shadowed */ DELETE FROM ${getShadowed(kind)} WHERE id = $1`);
  query(rawSql`/* many */ DELETE FROM ${getTooMany(kind)} WHERE id = $1`);
}
