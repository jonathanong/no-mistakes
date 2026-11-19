import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import * as missing from "./not-a-module";
import * as fragments from "./indexFragments";
import { AMBIENT_COLS, DEFAULT_COLS, RENAMED_COLS } from "./indexFragments";
import { QUAL_COLS } from "./indexQualFragments";
import { REQ_COLS } from "./indexRequireFragments";

export const unresolved = [missing];

async function renamed(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(RENAMED_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}
async function ambient(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(AMBIENT_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}
async function defaulted(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(DEFAULT_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}
async function required(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(REQ_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}
async function qualified(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(QUAL_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}
async function clean(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(fragments.CLEAN_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaRenamed(query: SessionExecutor, id: string) {
  return renamed(query, id);
}
export function viaAmbient(query: SessionExecutor, id: string) {
  return ambient(query, id);
}
export function viaDefaulted(query: SessionExecutor, id: string) {
  return defaulted(query, id);
}
export function viaRequired(query: SessionExecutor, id: string) {
  return required(query, id);
}
export function viaQualified(query: SessionExecutor, id: string) {
  return qualified(query, id);
}
export function viaClean(query: SessionExecutor, id: string) {
  return clean(query, id);
}
