import { query } from "@example/db";
import cryptoDefault, { randomBytes as bytes } from "node:crypto";
import * as cryptoNamespace from "crypto";
import foreignCrypto, { randomBytes as foreignBytes } from "other";
import type { randomBytes as typeBytes } from "node:crypto";

export function parameterShadow(bytes: typeof foreignBytes) {
  const table = `tmp_${bytes(8).toString("hex")}`;
  return query(`SELECT id FROM ${table}`);
}

export function receiverShadow(cryptoDefault: typeof foreignCrypto) {
  const table = `tmp_${cryptoDefault.randomBytes(8).toString("hex")}`;
  return query(`SELECT id FROM ${table}`);
}

export function namespaceShadow() {
  const cryptoNamespace = foreignCrypto;
  const table = `tmp_${cryptoNamespace.randomBytes(8).toString("hex")}`;
  return query(`SELECT id FROM ${table}`);
}

export function blockShadow() {
  {
    const table = `tmp_${bytes(8).toString("hex")}`;
    const bytes = foreignBytes;
    return query(`SELECT id FROM ${table}`);
  }
}

export function foreignImport() {
  const table = `tmp_${foreignBytes(8).toString("hex")}`;
  return query(`SELECT id FROM ${table}`);
}

export function foreignReceiver() {
  const table = `tmp_${foreignCrypto.randomBytes(8).toString("hex")}`;
  return query(`SELECT id FROM ${table}`);
}

export function typeOnly() {
  const table = `tmp_${typeBytes(8).toString("hex")}`;
  return query(`SELECT id FROM ${table}`);
}

export function unimported() {
  const table = `tmp_${crypto.randomBytes(8).toString("hex")}`;
  return query(`SELECT id FROM ${table}`);
}
