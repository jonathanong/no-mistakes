import { query } from "@example/db";
import { randomUUID } from "node:crypto";

export function unused(table: string) {
  return table.length;
}

export function stripped() {
  const table = `tmp_${randomUUID().replaceAll("-", "")}`;
  return query(`SELECT id FROM ${table}`);
}

export function regexReplace() {
  const table = "tmp_" + randomUUID().replace(/-/g, "");
  return query(`SELECT id FROM ${table}`);
}

export function cryptoUuid() {
  const table = `tmp_${crypto.randomUUID().replaceAll("-", "")}`;
  return query(`SELECT id FROM ${table}`);
}
