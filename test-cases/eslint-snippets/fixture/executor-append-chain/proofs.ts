import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

const COLS = sql`id, state`;
const LIMIT = 10;
const PAGE = sql` LIMIT ${LIMIT}`;

async function appendOnly(id: string, query: SessionExecutor) {
  await query(sql`SELECT `.append(COLS).append(sql` FROM widgets WHERE id = ${id}`));
}

export function j1(query: SessionExecutor, id: string) {
  return appendOnly(id, query);
}

async function inlineChain(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(sql`id`).append(sql` FROM widgets WHERE id = ${id}`));
}

async function pagedChain(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(COLS).append(sql` FROM widgets WHERE id = ${id}`).append(PAGE));
}

export function viaInline(query: SessionExecutor, id: string) {
  return inlineChain(query, id);
}
export function viaPaged(query: SessionExecutor, id: string) {
  return pagedChain(query, id);
}
