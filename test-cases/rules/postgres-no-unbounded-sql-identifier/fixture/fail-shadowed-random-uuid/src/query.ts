import { query } from "@example/db";
import crypto from "left-pad";

export function unused(table: string, label: string) {
  return table.length + label.length;
}

export function shadowed() {
  const randomUUID = () => process.env.T;
  const table = `tmp_${randomUUID().replaceAll("-", "")}`;
  return query(`SELECT id FROM ${table}`);
}

export function localCrypto() {
  const crypto = { randomUUID: () => process.env.T };
  const label = `tmp_${crypto.randomUUID().replace(/-/g, "")}`;
  return query(`SELECT id FROM ${label}`);
}

export function foreignCrypto() {
  const table = `tmp_${crypto.randomUUID().replaceAll("-", "")}`;
  return query(`SELECT id FROM ${table}`);
}
