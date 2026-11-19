import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const DYNAMIC_AWAIT = sql`id, state`;
export const DYNAMIC_THEN = sql`name`;
export const DYNAMIC_KEEP = sql`owner`;

async function awaitLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(DYNAMIC_AWAIT).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaDynamicAwait(query: SessionExecutor, id: string) {
  return awaitLocal(query, id);
}

async function thenLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(DYNAMIC_THEN).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaDynamicThen(query: SessionExecutor, id: string) {
  return thenLocal(query, id);
}

async function keepLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(DYNAMIC_KEEP).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaDynamicKeep(query: SessionExecutor, id: string) {
  return keepLocal(query, id);
}
