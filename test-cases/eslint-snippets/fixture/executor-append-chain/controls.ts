import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import { CALLED, LET_COLS, MUTATED, PARAMETERIZED } from "./fragments";

type Statement = ReturnType<typeof sql>;
const COLS = sql`id, state`;
const MUTABLE = sql`id, state`;
let REASSIGNED = sql`id, state`;
const RAW = "id, state";
const LIMIT = 10;
const COMPUTED = sql` LIMIT ${LIMIT + Math.random()}`;
declare function fragment(): Statement;

MUTABLE.append(sql`, extra`);
REASSIGNED = sql`id`;

async function appendedParameter(query: SessionExecutor, id: string, part: Statement) {
  await query(sql`SELECT `.append(part).append(sql` FROM widgets WHERE id = ${id}`));
}

async function appendedLet(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(REASSIGNED).append(sql` FROM widgets WHERE id = ${id}`));
}

async function appendedCall(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(fragment()).append(sql` FROM widgets WHERE id = ${id}`));
}

async function appendedMutated(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(MUTABLE).append(sql` FROM widgets WHERE id = ${id}`));
}

async function appendedRaw(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(RAW).append(sql` FROM widgets WHERE id = ${id}`));
}

async function appendedComputed(query: SessionExecutor, id: string) {
  await query(sql`SELECT id FROM widgets WHERE id = ${id}`.append(COMPUTED));
}

async function appendedLocalConst(query: SessionExecutor, id: string) {
  const inner = sql`id, state`;
  await query(sql`SELECT `.append(inner).append(sql` FROM widgets WHERE id = ${id}`));
}

async function constRoot(query: SessionExecutor, id: string) {
  await query(COLS.append(sql` FROM widgets WHERE id = ${id}`));
}

async function importedMutated(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(MUTATED).append(sql` FROM widgets WHERE id = ${id}`));
}

async function importedParameterized(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(PARAMETERIZED).append(sql` FROM widgets WHERE id = ${id}`));
}

async function importedLet(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(LET_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}

async function importedCalled(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(CALLED).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaParameter(query: SessionExecutor, id: string, part: Statement) {
  return appendedParameter(query, id, part);
}
export function viaLet(query: SessionExecutor, id: string) {
  return appendedLet(query, id);
}
export function viaCall(query: SessionExecutor, id: string) {
  return appendedCall(query, id);
}
export function viaMutated(query: SessionExecutor, id: string) {
  return appendedMutated(query, id);
}
export function viaRaw(query: SessionExecutor, id: string) {
  return appendedRaw(query, id);
}
export function viaComputed(query: SessionExecutor, id: string) {
  return appendedComputed(query, id);
}
export function viaLocalConst(query: SessionExecutor, id: string) {
  return appendedLocalConst(query, id);
}
export function viaConstRoot(query: SessionExecutor, id: string) {
  return constRoot(query, id);
}
export function viaImportedMutated(query: SessionExecutor, id: string) {
  return importedMutated(query, id);
}
export function viaImportedParameterized(query: SessionExecutor, id: string) {
  return importedParameterized(query, id);
}
export function viaImportedLet(query: SessionExecutor, id: string) {
  return importedLet(query, id);
}
export function viaImportedCalled(query: SessionExecutor, id: string) {
  return importedCalled(query, id);
}

async function importedUnresolved(query: SessionExecutor, id: string) {
  // @ts-expect-error the fragment is deliberately undeclared
  await query(sql`SELECT `.append(UNDECLARED).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaImportedUnresolved(query: SessionExecutor, id: string) {
  return importedUnresolved(query, id);
}
