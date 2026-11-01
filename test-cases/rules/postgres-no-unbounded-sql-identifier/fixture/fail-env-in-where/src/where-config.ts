import { query } from "@example/db";
import { config, literal, plain } from "./env-config";
import * as ns from "./env-config";

export function importedConfig(table: string, uid: string) {
  return query(`SELECT id FROM ${table} WHERE a = ${config.lim} AND id = ${uid}`);
}

export function importedPlain(table: string, uid: string) {
  return query(`SELECT id FROM ${table} WHERE a = ${plain} AND id = ${uid}`);
}

export function importedNamespace(table: string, uid: string) {
  return query(`SELECT id FROM ${table} WHERE a = ${ns.plain} AND id = ${uid}`);
}

export function importedLiteral(table: string, uid: string) {
  return query(`SELECT id FROM ${table} WHERE a = ${literal} AND id = ${uid}`);
}

export function namespaceLiteral(table: string, uid: string) {
  return query(`SELECT id FROM ${table} WHERE a = ${ns.literal} AND id = ${uid}`);
}

function loadCfg() {
  return { v: process.env.V };
}

export function helperConfig(table: string, uid: string) {
  const c = loadCfg();
  return query(`SELECT id FROM ${table} WHERE a = ${c.v} AND id = ${uid}`);
}

function loadLiteral() {
  return { v: "users" };
}

export function helperLiteral(table: string, uid: string) {
  const fixed = loadLiteral();
  return query(`SELECT id FROM ${table} WHERE a = ${fixed.v} AND id = ${uid}`);
}
