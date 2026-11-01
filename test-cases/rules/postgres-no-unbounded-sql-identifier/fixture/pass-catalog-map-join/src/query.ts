import { query } from "@example/db";
import { TABLES } from "./catalog";

export function joined() {
  const list = TABLES.join(", ");
  return query(`SELECT 1 FROM ${list}`);
}

export function mapped() {
  const refs = TABLES.map((table) => `public.${table}`).join(", ");
  return query(`SELECT 1 FROM ${refs}`);
}

export function pushed() {
  const parts: string[] = [];
  for (const table of TABLES) {
    parts.push(`public.${table}`);
  }
  return query(`SELECT 1 FROM ${parts.join(", ")}`);
}
