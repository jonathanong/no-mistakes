import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const ELEMENT_MUT = sql`id, state`;
export const ELEMENT_KEEP = sql`name`;

async function mutLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(ELEMENT_MUT).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaElementMut(query: SessionExecutor, id: string) {
  return mutLocal(query, id);
}

async function keepLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(ELEMENT_KEEP).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaElementKeep(query: SessionExecutor, id: string) {
  return keepLocal(query, id);
}
