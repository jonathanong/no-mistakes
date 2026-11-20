import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const EXTEND_COLS = sql`id, state`;
export const EXTEND_OTHER = sql`name`;

async function extendColsLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(EXTEND_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaExtendCols(query: SessionExecutor, id: string) {
  return extendColsLocal(query, id);
}

async function extendOtherLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(EXTEND_OTHER).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaExtendOther(query: SessionExecutor, id: string) {
  return extendOtherLocal(query, id);
}
