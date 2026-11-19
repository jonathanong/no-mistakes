import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const EQUALS_COLS = sql`id, state`;
export const EQUALS_KEEP = sql`name`;

async function equalsLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(EQUALS_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaEquals(query: SessionExecutor, id: string) {
  return equalsLocal(query, id);
}

async function keepLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(EQUALS_KEEP).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaEqualsKeep(query: SessionExecutor, id: string) {
  return keepLocal(query, id);
}
