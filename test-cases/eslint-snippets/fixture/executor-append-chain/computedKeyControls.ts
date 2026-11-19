import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const KEY_A = sql`id, state`;
export const KEY_B = sql`name`;

async function keyLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(KEY_A).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaKeyA(query: SessionExecutor, id: string) {
  return keyLocal(query, id);
}

async function otherLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(KEY_B).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaKeyB(query: SessionExecutor, id: string) {
  return otherLocal(query, id);
}
