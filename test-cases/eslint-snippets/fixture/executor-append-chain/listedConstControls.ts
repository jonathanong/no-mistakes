import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

const LISTED_COLS = sql`id, state`;

async function listedLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(LISTED_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaListedLocal(query: SessionExecutor, id: string) {
  return listedLocal(query, id);
}

export { LISTED_COLS };
