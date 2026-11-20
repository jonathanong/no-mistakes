import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const QUIET_COLS = sql`id, state`;
export const QUIET_OTHER = sql`name`;

async function quietColsLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(QUIET_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaQuietCols(query: SessionExecutor, id: string) {
  return quietColsLocal(query, id);
}

async function quietOtherLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(QUIET_OTHER).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaQuietOther(query: SessionExecutor, id: string) {
  return quietOtherLocal(query, id);
}
