import { query } from "@example/db";
import { randomUUID } from "node:crypto";

export function unused(table: string) {
  return table.length;
}

export function unstripped() {
  const table = `tmp_${randomUUID()}`;
  return query(`SELECT id FROM ${table}`);
}

export function firstHyphen() {
  const table = `tmp_${randomUUID().replace("-", "")}`;
  return query(`SELECT id FROM ${table}`);
}

export function suffixed(suffix: string) {
  return query(`SELECT id FROM ${`tmp_${randomUUID().replaceAll("-", "")}_${suffix}`}`);
}
