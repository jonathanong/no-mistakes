import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const TYPE_COLS = sql`id, state`;

async function typeLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(TYPE_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaTypeCols(query: SessionExecutor, id: string) {
  return typeLocal(query, id);
}
