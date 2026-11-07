import { query } from "@example/db";
import cryptoDefault, { randomBytes, randomBytes as bytes } from "node:crypto";
import * as cryptoNamespace from "crypto";

export function direct() {
  const table = `tmp_${randomBytes(8).toString("hex")}`;
  return query(`SELECT id FROM ${table}`);
}

export function alias() {
  const table = "tmp_" + bytes(8).toString("hex");
  return query(`SELECT id FROM ${table}`);
}

export function defaultReceiver() {
  const table = `tmp_${cryptoDefault.randomBytes(8).toString("hex")}`;
  return query(`SELECT id FROM ${table}`);
}

export function namespaceReceiver() {
  const table = `tmp_${cryptoNamespace.randomBytes(8).toString("hex")}_$42`;
  return query(`SELECT id FROM ${table}`);
}

export function zeroLength() {
  const table = `tmp_${bytes(0).toString("hex")}`;
  return query(`SELECT id FROM ${table}`);
}

export function wrappers() {
  const table = (`tmp_` as const) + (bytes((0x8 as number)).toString(`hex`) as string);
  return query(`SELECT id FROM ${table}`);
}
