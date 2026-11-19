import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const SHARED_COLS = sql`id, state`;

async function exportedLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(SHARED_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaExportedLocal(query: SessionExecutor, id: string) {
  return exportedLocal(query, id);
}
