import { query } from "@example/db";
import { randomBytes } from "node:crypto";

export function otherEncoding() {
  const table = `tmp_${randomBytes(8).toString("base64")}`;
  return query(`SELECT id FROM ${table}`);
}

export function dynamicSize(size: number) {
  const table = `tmp_${randomBytes(size).toString("hex")}`;
  return query(`SELECT id FROM ${table}`);
}

export function fractionalSize() {
  const table = `tmp_${randomBytes(1.5).toString("hex")}`;
  return query(`SELECT id FROM ${table}`);
}

export function callbackOverload() {
  const table = `tmp_${randomBytes(8, () => {}).toString("hex")}`;
  return query(`SELECT id FROM ${table}`);
}

export function noPrefix() {
  const table = `${randomBytes(8).toString("hex")}`;
  return query(`SELECT id FROM ${table}`);
}

export function unsafePrefix() {
  const table = `tmp-${randomBytes(8).toString("hex")}`;
  return query(`SELECT id FROM ${table}`);
}

export function unsafeSuffix() {
  const table = `tmp_${randomBytes(8).toString("hex")};`;
  return query(`SELECT id FROM ${table}`);
}

export function openSuffix(suffix: string) {
  const table = `tmp_${randomBytes(8).toString("hex")}_${suffix}`;
  return query(`SELECT id FROM ${table}`);
}

export function optionalCall() {
  const table = `tmp_${randomBytes?.(8).toString("hex")}`;
  return query(`SELECT id FROM ${table}`);
}
