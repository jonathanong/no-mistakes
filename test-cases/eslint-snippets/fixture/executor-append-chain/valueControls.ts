import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const VALUE_COLS = sql`id, state`;
export const VALUE_OTHER = sql`name`;

async function valueLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(VALUE_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaValue(query: SessionExecutor, id: string) {
  return valueLocal(query, id);
}

async function otherLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(VALUE_OTHER).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaValueOther(query: SessionExecutor, id: string) {
  return otherLocal(query, id);
}
