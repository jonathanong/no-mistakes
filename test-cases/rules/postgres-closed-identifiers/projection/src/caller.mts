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
