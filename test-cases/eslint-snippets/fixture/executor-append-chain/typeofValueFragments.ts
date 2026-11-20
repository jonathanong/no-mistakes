import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const KIND_COLS = sql`id, state`;
export const KIND_OTHER = sql`name`;

async function kindLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(KIND_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaKind(query: SessionExecutor, id: string) {
  return kindLocal(query, id);
}

async function kindOtherLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(KIND_OTHER).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaKindOther(query: SessionExecutor, id: string) {
  return kindOtherLocal(query, id);
}
