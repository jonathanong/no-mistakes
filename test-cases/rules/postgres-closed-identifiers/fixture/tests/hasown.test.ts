import { query } from "@example/db";
import { sql as rawSql } from "@example/raw-sql";

const relationTables = Object.freeze({
  vote: "users",
  tag: "posts",
} as const);

function getRelationTable(name: string) {
  if (!Object.hasOwn(relationTables, name)) throw new Error(`unknown ${name}`);
  return relationTables[name as keyof typeof relationTables];
}

const relationConfigs = Object.freeze({
  vote: Object.freeze({ table: "users", column: "user_id" }),
  tag: Object.freeze({ table: "posts", column: "post_id" }),
} as const);

function getConfig(name: string) {
  if (!Object.hasOwn(relationConfigs, name)) throw new Error(`unknown ${name}`);
  return relationConfigs[name as keyof typeof relationConfigs];
}

export function remove(kind: string) {
  return query(`DELETE FROM ${getRelationTable(kind)} WHERE id = $1`);
}

export function removeRaw(kind: string) {
  return query(rawSql`DELETE FROM ${getRelationTable(kind)} WHERE id = $1`);
}

export function one() {
  return query(rawSql`DELETE FROM ${getRelationTable("vote")} WHERE id = $1`);
}

export function correlated(kind: string) {
  const config = getConfig(kind);
  return query(rawSql`DELETE FROM ${config.table} WHERE ${config.column} = $1`);
}
