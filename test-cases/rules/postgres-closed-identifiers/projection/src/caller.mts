import { query } from "@example/db";
import { sql as rawSql } from "@example/raw-sql";
import { getConfig, getLoose, getRelationTable } from "./relations.mts";

export function remove(kind: string) {
  query(`/* both */ DELETE FROM ${getRelationTable(kind)} WHERE id = $1`);
}

export function one() {
  query(`/* one */ DELETE FROM ${getRelationTable("vote")} WHERE id = $1`);
}

export function correlated(kind: string) {
  const config = getConfig(kind);
  query(`/* corr */ DELETE FROM ${config.table} WHERE ${config.column} = $1`);
}

export function loose(kind: string) {
  query(`/* loose */ DELETE FROM ${getLoose(kind)} WHERE id = $1`);
}

export function rawRemove(kind: string) {
  query(rawSql`/* raw */ DELETE FROM ${getRelationTable(kind)} WHERE id = $1`);
}

export function literalConfig() {
  const config = getConfig("vote");
  query(`/* lit */ DELETE FROM ${config.table} WHERE ${config.column} = $1`);
}

export function boolConfig() {
  const flagged = getConfig(true);
  query(`/* bool */ DELETE FROM ${flagged.table} WHERE id = $1`);
}

export function reusedParameter(config: string) {
  query(`/* reused */ DELETE FROM ${config.table} WHERE id = $1`);
}

export function writtenConfig(kind: string) {
  const config = getConfig(kind);
  config.table = "accounts";
  query(`/* written-import */ DELETE FROM ${config.table} WHERE ${config.column} = $1`);
}
