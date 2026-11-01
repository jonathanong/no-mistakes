import { query } from "@example/db";

export function localColumn(table: string, uid: string) {
  const col = process.env.SCOL1;
  return query(`SELECT id FROM ${table} WHERE a = ${col} AND id = ${uid}`);
}

export function sameNameParameter(col: string) {
  return col;
}

export function localConfig(table: string, uid: string) {
  const cfg = { v: process.env.SCV };
  return query(`SELECT id FROM ${table} WHERE a = ${cfg.v} AND id = ${uid}`);
}

export function sameNameConfigParameter(cfg: string) {
  return cfg;
}

export function envDefault(table: string, d = process.env.SDEF) {
  return query(`SELECT id FROM ${table} WHERE a = ${d}`);
}
