import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const REFLECT_COLS = sql`id, state`;
export const REFLECT_OTHER = sql`name`;

async function reflectLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(REFLECT_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaReflect(query: SessionExecutor, id: string) {
  return reflectLocal(query, id);
}

async function otherLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(REFLECT_OTHER).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaReflectOther(query: SessionExecutor, id: string) {
  return otherLocal(query, id);
}
