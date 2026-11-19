import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const ASSIGN_COLS = sql`id, state`;
export const ASSIGN_OTHER = sql`name`;

async function assignLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(ASSIGN_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaAssign(query: SessionExecutor, id: string) {
  return assignLocal(query, id);
}

async function otherLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(ASSIGN_OTHER).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaAssignOther(query: SessionExecutor, id: string) {
  return otherLocal(query, id);
}
