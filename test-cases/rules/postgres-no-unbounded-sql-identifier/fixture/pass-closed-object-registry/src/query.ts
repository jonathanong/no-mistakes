import { query } from "@example/db";
import { FROZEN, TABLE_MAP } from "./registry";

const LOCAL_MAP = {
  user: "users",
  post: "posts",
} as const;

export function importedIndex(kind: string) {
  return query(`SELECT id FROM ${TABLE_MAP[kind]}`);
}

export function importedMember() {
  return query(`SELECT id FROM ${TABLE_MAP.user}`);
}

export function importedFrozen() {
  return query(`SELECT id FROM ${FROZEN.user}`);
}

export function localIndex(kind: string) {
  return query(`SELECT id FROM ${LOCAL_MAP[kind]}`);
}

export function localMember() {
  return query(`SELECT id FROM ${LOCAL_MAP.user}`);
}
